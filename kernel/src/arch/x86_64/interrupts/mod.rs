mod apic;
mod handlers;
mod idt;

use apic::{LVTEntry, LVTEntryFlags, APIC_BASE};
use core::arch::asm;
use idt::IDTDesc;

use crate::{
    arch::x86_64::acpi::{get_sdt, MADT},
    globals::paging_mapper,
    memory::{
        frame_allocator::Frame,
        paging::{EntryFlags, Page},
        PhysAddr,
    },
    println,
};

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

fn get_madt() -> &'static MADT {
    let sdt = get_sdt();
    unsafe { &*(sdt.get_entry_of_signatrue(*b"APIC").unwrap() as *const MADT) }
}

pub fn enable_apic_interrupts() {
    unsafe { asm!("sti") };

    let address = *APIC_BASE;
    println!("address of apic 0x{:x}", address);

    // mapping the apic address
    paging_mapper()
        .map_to(
            Page::containing_address(address),
            Frame::containing_address(address),
            EntryFlags::PRESENT | EntryFlags::WRITABLE,
        )
        .unwrap();

    let sivr = (address + 0xF0) as *mut u32;

    unsafe {
        core::ptr::write_volatile(sivr, 0x1ff);

        // let timer_addr = address + 0x320;
        // let timer = timer_addr as *mut LVTEntry;
        // core::ptr::write_unaligned(
        //     timer,
        //     LVTEntry {
        //         flags: LVTEntryFlags::TIMER_PERIODIC,
        //         entry: 0x20,
        //         unused: 0,
        //     },
        // );
        //
        // let divide_reg = address + 0x3E0;
        // let divide_reg = divide_reg as *mut u8;
        // *divide_reg = 0x0000000B;
        //
        // let init_reg = address + 0x380;
        // let init_reg = init_reg as *mut u32;
        // *init_reg = 0xFFFFFFF;
        //
        // let timer = *(timer_addr as *mut LVTEntry);
        // let flags = timer.flags;
        // let timer = timer.entry as u64 | ((flags.bits() as u64) << 7);
        let madt = get_madt();
        println!("{:#?}", madt);
    }
}
