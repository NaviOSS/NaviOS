use spin::Mutex;

use crate::{
    memory::{
        allocator::LinkedListAllocator,
        frame_allocator::RegionAllocator,
        paging::{self, Mapper},
    },
    terminal::framebuffer::Terminal,
    threading::Scheduler,
    utils::Locked,
};

pub static mut SCHEDULER: Option<Scheduler> = None;

pub fn scheduler() -> &'static mut Scheduler {
    unsafe { SCHEDULER.as_mut().unwrap() }
}

pub fn scheduler_inited() -> bool {
    unsafe { SCHEDULER.is_some() }
}
// globals are initialized using the kinit function below is there definition and getters
pub static mut FRAME_ALLOCATOR: Option<RegionAllocator> = None;

pub fn frame_allocator() -> &'static mut RegionAllocator {
    unsafe { FRAME_ALLOCATOR.as_mut().unwrap() }
}

pub static mut TERMINAL: Option<Terminal<'static>> = None;
pub fn terminal_inited() -> bool {
    unsafe { TERMINAL.is_some() }
}

pub fn terminal() -> &'static mut Terminal<'static> {
    unsafe { TERMINAL.as_mut().unwrap() }
}

pub static mut PAGING_MAPPER: Option<paging::Mapper> = None;
pub fn paging_mapper() -> &'static mut Mapper {
    unsafe { PAGING_MAPPER.as_mut().unwrap() }
}

// Some in x86 family
pub static mut RSDP_ADDR: Option<u64> = None;
pub fn rsdp_addr() -> u64 {
    unsafe { RSDP_ADDR.unwrap() }
}

// safer globals that uses the locked type!
// still has to be init'ed
#[global_allocator]
static GLOBAL_ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());

pub fn global_allocator() -> &'static Mutex<LinkedListAllocator> {
    &GLOBAL_ALLOCATOR.inner
}
