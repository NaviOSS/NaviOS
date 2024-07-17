mod handlers;
mod idt;

use core::arch::asm;
use idt::IDTDesc;

pub fn init_idt() {
    unsafe {
        asm!("lidt [{}]", in(reg) &*IDTDesc, options(nostack));
    }
}

pub fn enable_interrupts() {
    unsafe {
        asm!("sti");
    }
}
