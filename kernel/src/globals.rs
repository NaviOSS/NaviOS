#![allow(static_mut_refs)]
use spin::Mutex;

use crate::{
    memory::{allocator::LinkedListAllocator, frame_allocator::RegionAllocator},
    terminal::framebuffer::Terminal,
    threading::Scheduler,
    utils::{elf::Elf, Locked},
};

/// boot info
#[derive(Debug)]
pub struct Kernel {
    pub frame_allocator: RegionAllocator,

    pub phy_offset: usize,
    pub rsdp_addr: Option<u64>,
    pub elf: Elf<'static>,
}

impl Kernel {
    // TODO: lock the frame_allocator!!!
    #[inline]
    pub fn frame_allocator(&'static mut self) -> &'static mut RegionAllocator {
        &mut self.frame_allocator
    }
}
pub static mut KERNEL: Option<Kernel> = None;

pub fn kernel() -> &'static mut Kernel {
    unsafe { KERNEL.as_mut().unwrap() }
}
pub fn kernel_inited() -> bool {
    unsafe { KERNEL.is_some() }
}

pub static mut TERMINAL: Option<Terminal> = None;
pub fn terminal_inited() -> bool {
    unsafe { TERMINAL.is_some() }
}

pub fn terminal() -> &'static mut Terminal {
    unsafe { TERMINAL.as_mut().unwrap() }
}

pub static mut SCHEDULER: Option<Scheduler> = None;
pub fn scheduler_inited() -> bool {
    unsafe { SCHEDULER.is_some() }
}

pub fn scheduler() -> &'static mut Scheduler {
    unsafe { SCHEDULER.as_mut().unwrap() }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());

pub fn global_allocator() -> &'static Mutex<LinkedListAllocator> {
    &GLOBAL_ALLOCATOR.inner
}
