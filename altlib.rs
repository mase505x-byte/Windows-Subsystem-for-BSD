use std::ptr;
use std::slice;

pub struct ExecutableRegion {
    ptr: *mut u8,
    size: usize,
}

impl ExecutableRegion {
    pub fn from_bytes(code: &[u8]) -> Result<Self, &'static str> {
        if code.is_empty() {
            return Err("Empty instruction buffer");
        }

        let page_size = 4096usize;
        let alloc_size = (code.len() + page_size - 1) & !(page_size - 1);

        unsafe {
            let ptr = libc::mmap(
                ptr::null_mut(),
                alloc_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            );

            if ptr == libc::MAP_FAILED {
                return Err("mmap allocation failed");
            }

            ptr::copy_nonoverlapping(code.as_ptr(), ptr as *mut u8, code.len());

            if libc::mprotect(ptr, alloc_size, libc::PROT_READ | libc::PROT_EXEC) != 0 {
                libc::munmap(ptr, alloc_size);
                return Err("mprotect W^X transition failed");
            }

            Ok(Self {
                ptr: ptr as *mut u8,
                size: alloc_size,
            })
        }
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }
}

impl Drop for ExecutableRegion {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.ptr as *mut libc::c_void, self.size);
        }
    }
}

pub struct BinaricAssembler {
    buffer: Vec<u8>,
}

impl BinaricAssembler {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(256),
        }
    }

    pub fn emit_u8(&mut self, byte: u8) {
        self.buffer.push(byte);
    }

    pub fn emit_bytes(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    pub fn emit_u32(&mut self, val: u32) {
        self.buffer.extend_from_slice(&val.to_le_bytes());
    }

    pub fn emit_u64(&mut self, val: u64) {
        self.buffer.extend_from_slice(&val.to_le_bytes());
    }

    pub fn patch_u32(&mut self, offset: usize, val: u32) -> Result<(), &'static str> {
        if offset + 4 > self.buffer.len() {
            return Err("Patch offset out of bounds");
        }
        let bytes = val.to_le_bytes();
        self.buffer[offset..offset + 4].copy_from_slice(&bytes);
        Ok(())
    }

    pub fn current_offset(&self) -> usize {
        self.buffer.len()
    }

    pub fn emit_sysv_to_win64_trampoline(&mut self, win64_target: u64) {
        self.emit_bytes(&[0x55]);
        self.emit_bytes(&[0x48, 0x89, 0xE5]);

        self.emit_bytes(&[0x49, 0x89, 0xCA]);
        self.emit_bytes(&[0x48, 0x89, 0xF9]);
        self.emit_bytes(&[0x48, 0x89, 0xF2]);
        self.emit_bytes(&[0x4D, 0x89, 0xC0]);
        self.emit_bytes(&[0x4D, 0x89, 0xC9]);

        self.emit_bytes(&[0x48, 0x83, 0xEC, 0x20]);

        self.emit_bytes(&[0x48, 0xB8]);
        self.emit_u64(win64_target);

        self.emit_bytes(&[0xFF, 0xD0]);

        self.emit_bytes(&[0x48, 0x83, 0xC4, 0x20]);

        self.emit_bytes(&[0x5D]);
        self.emit_bytes(&[0xC3]);
    }

    pub fn emit_ret(&mut self) {
        self.emit_u8(0xC3);
    }

    pub fn finalize(self) -> Result<ExecutableRegion, &'static str> {
        ExecutableRegion::from_bytes(&self.buffer)
    }
}

#[no_mangle]
pub extern "C" fn wsb_asm_create() -> *mut BinaricAssembler {
    Box::into_raw(Box::new(BinaricAssembler::new()))
}

#[no_mangle]
pub unsafe extern "C" fn wsb_asm_emit_trampoline(
    asm: *mut BinaricAssembler,
    win64_fn_addr: u64,
) {
    if !asm.is_null() {
        (*asm).emit_sysv_to_win64_trampoline(win64_fn_addr);
    }
}

#[no_mangle]
pub unsafe extern "C" fn wsb_asm_patch_u32(
    asm: *mut BinaricAssembler,
    offset: usize,
    value: u32,
) -> i32 {
    if asm.is_null() {
        return -1;
    }
    match (*asm).patch_u32(offset, value) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn wsb_asm_finalize(
    asm: *mut BinaricAssembler,
) -> *mut ExecutableRegion {
    if asm.is_null() {
        return ptr::null_mut();
    }
    let assembler = Box::from_raw(asm);
    match assembler.finalize() {
        Ok(region) => Box::into_raw(Box::new(region)),
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn wsb_region_entry(region: *const ExecutableRegion) -> *const u8 {
    if region.is_null() {
        return ptr::null();
    }
    (*region).as_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn wsb_region_free(region: *mut ExecutableRegion) {
    if !region.is_null() {
        drop(Box::from_raw(region));
    }
}

#[no_mangle]
pub unsafe extern "C" fn wsb_asm_emit_raw(
    asm: *mut BinaricAssembler,
    bytes: *const u8,
    len: usize,
) -> i32 {
    if asm.is_null() || bytes.is_null() {
        return -1;
    }
    let slice = slice::from_raw_parts(bytes, len);
    (*asm).emit_bytes(slice);
    0
}
