use core::arch::asm;


#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct CPUStatus {
    pub rflags: u64,
    pub ss: u64,
    pub cs: u64,

    pub rip: u64,

    rbx: u64,
    rax: u64,
    pub rsp: u64
}


impl CPUStatus {
    pub extern "C" fn save_inner(self) -> Self {
        self
    }

    pub extern "C" fn save() -> Self {
        unsafe {        
            asm!("
            push rsp
            push rax
            push rbx
            push 0
            push 0x8
            push 0x10
            pushfq 
            call {}
            add rsp, 0x40
            ret
            ", sym Self::save_inner, options(noreturn))
        }
    }
    
    /// saves the current cpu status expect for the rip it instead uses a provided address
    #[inline]
    pub fn save_with_address(address: usize) -> Self {
        let mut captured = Self::save();
        captured.rip = address as u64;
        captured
    }

    #[inline]
    pub fn restore(self) -> ! {
        unsafe {
            asm!("
            mov rax, {}
            mov rbx, {}
            
            mov rsp, {}
            push {}
            popfq
            jmp {}
            ", 
            in(reg) self.rax, 
            in(reg) self.rbx, 
            in(reg) self.rsp, 
            in(reg) self.rflags,            
            in(reg) self.rip, options(noreturn))
        }
    }}

