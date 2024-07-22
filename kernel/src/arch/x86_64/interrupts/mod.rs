mod apic;
mod handlers;
mod idt;

use apic::{
    get_io_apic_addr, get_local_apic_addr, get_local_apic_reg, get_madt, write_ioapic_irq,
    LVTEntry, LVTEntryFlags, IOREDTBL,
};
use core::arch::asm;
use idt::IDTDesc;

use crate::{memory::PhysAddr, println};

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

pub fn enable_apic_interrupts() {
    unsafe { asm!("sti") };

    let address = get_local_apic_addr();
    let sivr = get_local_apic_reg(address, 0xF0) as *mut u32;

    unsafe {
        core::ptr::write_volatile(sivr, 0x1ff);

        let madt = get_madt();
        let ioapic_addr = get_io_apic_addr(madt);
        let apic_id = *(get_local_apic_reg(address, 0x20) as *const u8);

        println!("id: {apic_id}");
        let keyboard = IOREDTBL::new(
            LVTEntry {
                entry: 0x21,
                flags: LVTEntryFlags::empty(),
            },
            apic_id,
        );

        write_ioapic_irq(ioapic_addr, 1, keyboard);
    }
}
