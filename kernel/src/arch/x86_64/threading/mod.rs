use core::arch::asm;

use crate::{println, serial};
use core::mem::size_of;

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct CPUStatus {
    ss: u64,
    cs: u64,

    rsp: u64,
    rip: u64,

    rbx: u64,
    rax: u64
}


impl CPUStatus {
    extern "C" fn save_inner(self) -> Self {
        serial!("{:#?}", self.clone());
        unsafe {
            asm!("add rsp, {}", in(reg) size_of::<CPUStatus>())
        }
        self
    }

    #[inline]
    pub fn save(insturaction: usize, stack_segment: usize, code_segment: usize, stack_pointer: usize) -> Self {        
        unsafe {
            asm!("
            push rax
            push rbx
            
            push {}
            push {}

            push {}
            push {}
            push 0
            jmp {}
            ",  
            in(reg) insturaction, 
            in(reg) stack_pointer, 
            in(reg) code_segment, 
            in(reg) stack_segment, 
            sym Self::save_inner, options(noreturn));
        }
    }
    
    /// saves the current status expect for rip
    #[inline]
    pub fn save_with_address(instruaction: usize) -> Self {
        let rsp;
        unsafe {
            asm!("
            mov {}, rsp
            ", 
            out(reg) rsp,
            options(nostack, nomem))
        }

        let save  = Self::save(instruaction, 0x10, 0x8, rsp);
        save
    }

    
    #[inline]
    pub fn restore(self) -> ! {
        unsafe {
            asm!("
            mov rax, {}
            mov rbx, {}
            
            mov rsp, {}
            jmp {}
             
            ", 
            in(reg) self.rax, 
            in(reg) self.rbx, 
            in(reg) self.rsp, 
            in(reg) self.rip, options(noreturn))
        }
    }}

