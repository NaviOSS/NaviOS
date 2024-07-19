mod apic;
mod handlers;
mod idt;

use apic::{LVTEntry, LVTEntryFlags};
use core::arch::asm;
use idt::IDTDesc;

use crate::{
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

pub fn enable_apic_interrupts() {
    let apic_base = read_msr(0x1B);
    let address = apic_base & 0xFFFFF000;
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
        core::ptr::write_volatile(sivr, 0xff);
    }

    let timer_addr = address + 0x320;
    let timer = timer_addr as *mut LVTEntry;
    unsafe {
        core::ptr::write_unaligned(
            timer,
            LVTEntry {
                flags: LVTEntryFlags::TIMER_PERIODIC,
                entry: 32,
                unused: 0,
            },
        );

        let timer = *(timer_addr as *mut LVTEntry);
        println!("{:#?}", timer);
    }
}
