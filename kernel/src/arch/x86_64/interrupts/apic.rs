use bitflags::bitflags;

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
        const LEVEL_TRIGGERED = 1 << 8;
        const DISABLED = 1 << 9;
        const TIMER_PERIODIC = 1 << 11;
    }
}
