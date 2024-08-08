use core::arch::asm;
#[derive(Debug, Clone, Copy, Default)]
pub struct CPUStatus {
    rax: u64,
    rbx: u64,

    rip: u64,
    rsp: u64,

    cs: u64,
    ss: u64,
}

#[macro_export]
macro_rules! push_general {
    () => {
        unsafe {
        asm!("
        push rax
        push rbx
        ")
        }
    };
}

#[macro_export]
macro_rules! pop_general {
    () => {
        unsafe {
        asm!("
        pop rbx
        pop rax
        ")
        }
    };
}
impl CPUStatus {
    #[inline]
    pub fn save(insturaction: usize, stack_segment: usize, code_segment: usize, stack_pointer: usize) -> Self {
        pop_general!();
        let (rax, rbx, rip, rsp, ss, cs);

        unsafe {
            asm!(
                "
        mov {}, rax
        mov {}, rbx

        ", 
        out(reg) rax, 
        out(reg) rbx,
        options(nostack, nomem))
        }

        rip = insturaction as u64;
        rsp = stack_pointer as u64;

        ss = stack_segment as u64;
        cs = code_segment as u64;
        
        CPUStatus {
            rax,
            rbx,

            rip,
            rsp,

            ss,
            cs
        }
    }
    
    /// saves the current status expect for rip
    #[inline]
    pub fn save_with_address(instruaction: usize) -> Self {
        let (ss, cs, rsp);

        unsafe {
            asm!("
            mov {}, rsp
            mov {}, ss
            mov {}, cs
            ", 
            out(reg) rsp,

            out(reg) ss,
            out(reg) cs,

            options(nostack, nomem)) }

        Self::save(instruaction, ss, cs, rsp)
    
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

