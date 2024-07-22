mod acpi;
mod gdt;
mod interrupts;

use core::arch::asm;

use interrupts::{enable_apic_interrupts, init_idt};

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

pub extern "C" fn init() {
    init_gdt();
    init_idt();
    enable_apic_interrupts();
}

#[macro_export]
macro_rules! arch_init {
    () => {
        use arch::x86_64::init;
        init()
    };
}
