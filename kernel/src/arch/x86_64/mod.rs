mod acpi;
mod gdt;
mod interrupts;

use core::arch::asm;

use interrupts::{apic, init_idt};

use crate::memory;

use self::gdt::init_gdt;

pub fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe {
        asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack, preserves_flags));
    }
    value
}

pub fn outb(port: u16, value: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
    }
}

pub fn init() {
    init_gdt();
    init_idt();

    unsafe { memory::init_memory().unwrap() }
    apic::enable_apic_interrupts();
}

#[macro_export]
macro_rules! arch_init {
    () => {
        arch::x86_64::init()
    };
}
