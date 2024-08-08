mod acpi;
mod gdt;
pub mod interrupts;
pub mod serial;
pub mod threading;

use core::arch::asm;

use interrupts::{apic, init_idt};

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
#[inline]
pub fn init() {
    init_gdt();
    init_idt();

    apic::enable_apic_interrupts();
}

#[macro_export]
macro_rules! arch_init {
    () => {
        arch::x86_64::init()
    };
}
