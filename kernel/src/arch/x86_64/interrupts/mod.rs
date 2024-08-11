pub mod apic;
pub mod handlers;
mod idt;

use core::arch::asm;
use idt::IDTDesc;

use crate::PhysAddr;

#[derive(Debug)]
#[repr(C, packed)]
pub struct InterruptFrame {
    pub insturaction: u64,
    pub code_segment: u64,
    pub flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct TrapFrame {
    pub insturaction: u64,
    pub code_segment: u64,
    pub flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
    error_code: u64,
}

pub fn read_msr(msr: u32) -> PhysAddr {
    let (low, high): (u32, u32);
    unsafe {
        asm!(
            "
            mov ecx, {0:e}
            rdmsr
            mov {1:e}, eax
            mov {2:e}, edx
            ", 
            in(reg) msr, out(reg) low, out(reg) high
        );
    }

    (high as usize) << 32 | (low as usize)
}

pub fn init_idt() {
    unsafe {
        asm!("lidt [{}]", in(reg) &*IDTDesc, options(nostack));
    }
}
