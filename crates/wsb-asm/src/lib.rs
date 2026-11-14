//! Minimal x86-64 byte assembler for inspection and controlled ABI-stub generation.
//! This crate emits bytes only: it never allocates executable memory or runs generated code.
use std::ptr;

#[derive(Clone, Debug, Default)]
pub struct BinaricAssembler { buffer: Vec<u8> }

impl BinaricAssembler {
    pub fn new() -> Self { Self::default() }
    pub fn emit_u8(&mut self, value: u8) { self.buffer.push(value); }
    pub fn emit_bytes(&mut self, bytes: &[u8]) { self.buffer.extend_from_slice(bytes); }
    pub fn emit_u32(&mut self, value: u32) { self.emit_bytes(&value.to_le_bytes()); }
    pub fn emit_u64(&mut self, value: u64) { self.emit_bytes(&value.to_le_bytes()); }
    pub fn current_offset(&self) -> usize { self.buffer.len() }
    pub fn as_bytes(&self) -> &[u8] { &self.buffer }

    pub fn patch_u32(&mut self, offset: usize, value: u32) -> Result<(), &'static str> {
        let end = offset.checked_add(4).ok_or("patch offset overflow")?;
        let slot = self.buffer.get_mut(offset..end).ok_or("patch offset out of bounds")?;
        slot.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    /// Emit a SysV-to-Microsoft-x64 call stub for the first four integer/pointer arguments.
    /// Additional stack arguments and floating-point arguments are deliberately unsupported.
    /// The caller must only invoke a target with a matching signature on a compatible x86-64 host.
    pub fn emit_sysv_to_win64_trampoline(&mut self, target: u64) {
        // SysV: rdi,rsi,rdx,rcx -> Win64: rcx,rdx,r8,r9.
        self.emit_bytes(&[
            0x48, 0x83, 0xEC, 0x28, // sub rsp, 40: shadow space + call alignment
            0x4D, 0x89, 0xC1,       // mov r9, r8
            0x49, 0x89, 0xD0,       // mov r8, rdx
            0x48, 0x89, 0xF2,       // mov rdx, rsi
            0x48, 0x89, 0xF9,       // mov rcx, rdi
            0x48, 0xB8,             // mov rax, imm64
        ]);
        self.emit_u64(target);
        self.emit_bytes(&[
            0xFF, 0xD0,             // call rax
            0x48, 0x83, 0xC4, 0x28, // add rsp, 40
            0xC3,                   // ret
        ]);
    }
}

/// C ABI: create an empty assembler. Release it with `wsb_asm_free`.
#[no_mangle]
pub extern "C" fn wsb_asm_create() -> *mut BinaricAssembler {
    Box::into_raw(Box::new(BinaricAssembler::new()))
}

/// # Safety
/// `asm` must be a live pointer returned by `wsb_asm_create`, or null.
#[no_mangle]
pub unsafe extern "C" fn wsb_asm_free(asm: *mut BinaricAssembler) {
    if !asm.is_null() { drop(Box::from_raw(asm)); }
}

/// # Safety
/// `asm` must be a live pointer returned by `wsb_asm_create`, or null.
#[no_mangle]
pub unsafe extern "C" fn wsb_asm_emit_trampoline(asm: *mut BinaricAssembler, target: u64) -> i32 {
    if asm.is_null() { return -1; }
    (*asm).emit_sysv_to_win64_trampoline(target);
    0
}

/// # Safety
/// `asm` must be live. `out` must have room for the complete byte sequence when non-null.
/// Pass null as `out` to query the required length; length is always written when non-null.
#[no_mangle]
pub unsafe extern "C" fn wsb_asm_copy_bytes(asm: *const BinaricAssembler, out: *mut u8, capacity: usize, length: *mut usize) -> i32 {
    if asm.is_null() || length.is_null() { return -1; }
    let bytes = &(*asm).as_bytes();
    *length = bytes.len();
    if out.is_null() { return 0; }
    if capacity < bytes.len() { return -2; }
    ptr::copy_nonoverlapping(bytes.as_ptr(), out, bytes.len());
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn patches_only_existing_bytes() {
        let mut asm = BinaricAssembler::new();
        asm.emit_u32(1);
        asm.patch_u32(0, 0x12345678).unwrap();
        assert_eq!(asm.as_bytes(), &0x12345678u32.to_le_bytes());
        assert!(asm.patch_u32(1, 0).is_err());
        assert!(asm.patch_u32(usize::MAX, 0).is_err());
    }
    #[test]
    fn trampoline_contains_target_and_return() {
        let target = 0x1122334455667788;
        let mut asm = BinaricAssembler::new();
        asm.emit_sysv_to_win64_trampoline(target);
        assert!(asm.as_bytes().windows(8).any(|w| w == target.to_le_bytes()));
        assert_eq!(asm.as_bytes().last(), Some(&0xC3));
    }
}
