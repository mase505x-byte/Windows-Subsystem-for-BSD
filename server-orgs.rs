
use std::mem;
use std::ptr;

#[cfg(windows)]
extern "system" {
    fn VirtualAlloc(lpAddress: *mut u8, dwSize: usize, flAllocationType: u32, flProtect: u32) -> *mut u8;
    fn VirtualProtect(lpAddress: *mut u8, dwSize: usize, flNewProtect: u32, lpflOldProtect: *mut u32) -> i32;
    fn FlushInstructionCache(hProcess: isize, lpBaseAddress: *const u8, dwSize: usize) -> i32;
    fn GetCurrentProcess() -> isize;
}

const MEM_COMMIT: u32 = 0x1000;
const MEM_RESERVE: u32 = 0x2000;
const PAGE_READWRITE: u32 = 0x04;
const PAGE_EXECUTE_READ: u32 = 0x20;

pub struct MutableBlock {
    ptr: *mut u8,
    size: usize,
    imm_offset: usize,
}

impl MutableBlock {
    pub unsafe fn new(initial_val: u32) -> Self {
        let size = 4096;
        let ptr = VirtualAlloc(ptr::null_mut(), size, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
        let mut code = vec![0xB8];
        code.extend_from_slice(&initial_val.to_le_bytes());
        code.push(0xC3);

        ptr::copy_nonoverlapping(code.as_ptr(), ptr, code.len());

        let mut old_protect = 0;
        VirtualProtect(ptr, size, PAGE_EXECUTE_READ, &mut old_protect);
        FlushInstructionCache(GetCurrentProcess(), ptr, code.len());

        Self { ptr, size, imm_offset: 1 }
    }
    pub unsafe fn mutate_value(&self, new_val: u32) {
        let mut old_protect = 0;
        VirtualProtect(self.ptr, self.size, PAGE_READWRITE, &mut old_protect);
        
        let target = self.ptr.add(self.imm_offset) as *mut u32;
        ptr::write_unaligned(target, new_val);
		
        VirtualProtect(self.ptr, self.size, PAGE_EXECUTE_READ, &mut old_protect);
        FlushInstructionCache(GetCurrentProcess(), self.ptr, 6);
    }

    pub unsafe fn execute(&self) -> u32 {
        let func: extern "C" fn() -> u32 = mem::transmute(self.ptr);
        func()
    }
}
