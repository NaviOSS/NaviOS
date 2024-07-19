use super::read_msr;
use bitflags::bitflags;
use lazy_static::lazy_static;

use crate::PhysAddr;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct LVTEntry {
    pub entry: u8,
    pub flags: LVTEntryFlags,
    pub unused: u8,
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct LVTEntryFlags: u16 {
        const LEVEL_TRIGGERED = 1 << 7;
        const DISABLED = 1 << 8;
        const TIMER_PERIODIC = 1 << 9;
    }
}
lazy_static! {
    pub static ref APIC_BASE: PhysAddr = read_msr(0x1B) & 0xFFFFF000;
}

#[inline]
pub fn send_eoi() {
    unsafe {
        let eoi_reg = *APIC_BASE + 0xB0;
        let eoi_reg = eoi_reg as *mut u32;
        *eoi_reg = 0;
    }
}
