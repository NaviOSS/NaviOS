mod acpi;
pub mod gdt;
pub mod interrupts;
pub mod power;
pub mod serial;
pub(self) mod syscalls;
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
pub fn enable_sse() {
    unsafe {
        asm!(
            "
            mov rax, cr0
            and ax, 0xFFFB
            or ax, 0x2
            mov cr0, rax
            mov rax, cr4
            or ax, 3 << 9
            mov cr4, rax
        ",
            options(nostack)
        )
    }
}

/// simple init less likely to panic
/// highly required
#[inline]
pub fn init_phase1() {
    init_serial();
    init_gdt();
    init_idt();
}
/// complexer init
#[inline]
pub fn init_phase2() {
    acpi::enable_acpi(FADT::get(get_sdt()));
    apic::enable_apic_interrupts();
    enable_sse();
}
