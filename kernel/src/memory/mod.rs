pub mod allocator;
pub mod frame_allocator;
pub mod paging;

// types for better code reability
pub type VirtAddr = usize;
pub type PhysAddr = usize;

use frame_allocator::Frame;
use paging::{EntryFlags, MapToError, Page};

use crate::{
    globals::{frame_allocator, global_allocator, paging_mapper},
    serial,
};

#[inline]
pub fn map_present(addr: PhysAddr) {
    paging_mapper()
        .map_to(
            Page::containing_address(addr),
            Frame::containing_address(addr),
            EntryFlags::PRESENT,
        )
        .unwrap();
}

#[inline]
pub fn map_writeable(addr: PhysAddr) {
    paging_mapper()
        .map_to(
            Page::containing_address(addr),
            Frame::containing_address(addr),
            EntryFlags::PRESENT | EntryFlags::WRITABLE,
        )
        .unwrap();
}

fn p4_index(addr: VirtAddr) -> usize {
    (addr >> 39) & 0x1FF
}
fn p3_index(addr: VirtAddr) -> usize {
    (addr >> 30) & 0x1FF
}
fn p2_index(addr: VirtAddr) -> usize {
    (addr >> 21) & 0x1FF
}
fn p1_index(addr: VirtAddr) -> usize {
    (addr >> 12) & 0x1FF
}

pub fn translate(addr: VirtAddr) -> (PhysAddr, usize, usize, usize, usize) {
    (
        addr & 0xFFF,
        p1_index(addr),
        p2_index(addr),
        p3_index(addr),
        p4_index(addr),
    )
}

pub const fn align_up(address: usize, alignment: usize) -> usize {
    (address + alignment - 1) & !(alignment - 1)
}

pub const fn align_down(x: usize, alignment: usize) -> usize {
    x & !(alignment - 1)
}

pub const INIT_HEAP_SIZE: usize = 4 * 9 * 1024 * 1024;

// TODO! make the memory module more generic for different architectures; for now we can only support x86_64 because of the bootloader crate so take into account making our own bootloader for aarch64
pub unsafe fn init_memory(heap_start: usize) -> Result<(), MapToError> {
    serial!(
        "initing the heap... 0x{:x}..0x{:x}\n",
        heap_start,
        heap_start + INIT_HEAP_SIZE
    );
    let page_range = {
        let heap_start = heap_start;
        let heap_end = heap_start + INIT_HEAP_SIZE - 1;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::iter_pages(heap_start_page, heap_end_page)
    };
    serial!("Iter created!\n");

    let flags = EntryFlags::PRESENT | EntryFlags::WRITABLE;
    for page in page_range {
        let frame = frame_allocator()
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        unsafe { paging_mapper().map_to(page, frame, flags)?.flush() };
    }

    global_allocator().lock().init(heap_start, INIT_HEAP_SIZE);
    serial!("init done\n");
    Ok(())
}
