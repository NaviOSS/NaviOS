mod acpi;
mod gdt;
pub mod interrupts;
pub mod power;
pub mod serial;
pub mod threading;

use core::arch::asm;

use acpi::{get_sdt, FADT};
use interrupts::{apic, init_idt};
use serial::init_serial;

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

pub fn outw(port: u16, value: u16) {
    unsafe {
        asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack, preserves_flags));
    }
}

pub fn inw(port: u16) -> u16 {
    let value;
    unsafe {
        asm!("in ax, dx", out("ax") value, in("dx") port, options(nomem, nostack, preserves_flags));
    }
    value
}

#[inline]
pub fn init() {
    init_serial();
    init_gdt();
    init_idt();

    acpi::enable_acpi(FADT::get(get_sdt()));
    apic::enable_apic_interrupts();
}
