use spin::Mutex;

use crate::{
    memory::{
        allocator::LinkedListAllocator,
        frame_allocator::RegionAllocator,
        paging::{self, Mapper},
    },
    terminal::framebuffer::Terminal,
    utils::Locked,
};

// globals are initialized using the kinit function below is there definition and getters
pub static mut FRAME_ALLOCATOR: Option<RegionAllocator> = None;

pub fn frame_allocator() -> &'static mut RegionAllocator {
    unsafe { FRAME_ALLOCATOR.as_mut().unwrap() }
}

pub static mut TERMINAL: Option<Terminal<'static>> = None;

pub fn terminal() -> &'static mut Terminal<'static> {
    unsafe { TERMINAL.as_mut().unwrap() }
}

pub static mut PAGING_MAPPER: Option<paging::Mapper> = None;
pub fn paging_mapper() -> &'static mut Mapper {
    unsafe { PAGING_MAPPER.as_mut().unwrap() }
}

// safer globals that uses the locked type!
// still has to be init'ed
#[global_allocator]
static GLOBAL_ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());

pub fn global_allocator() -> &'static Mutex<LinkedListAllocator> {
    &GLOBAL_ALLOCATOR.inner
}
