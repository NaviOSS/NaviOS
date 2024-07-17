pub mod allocator;
pub mod frame_allocator;
pub mod page;

// types for better code reability
pub type VirtAddr = usize;
pub type PhysAddr = usize;

use bootloader_api::info::{MemoryRegions, Optional};
use page::{EntryFlags, MapToError, Mapper, Page};

use crate::{println, utils::Locked};

use self::frame_allocator::RegionAllocator;

pub const fn align(addr: usize, align: usize) -> usize {
    let remainder = addr % align;
    if remainder == 0 {
        addr
    } else {
        addr - remainder + align
    }
}

pub const fn align_up(x: usize, alignment: usize) -> usize {
    (x + alignment - 1) & !(alignment - 1)
}

pub const fn align_down(x: usize, alignment: usize) -> usize {
    x & !(alignment - 1)
}

#[global_allocator]
pub static GLOBAL_ALLOCATOR: Locked<allocator::LinkedListAllocator> =
    Locked::new(allocator::LinkedListAllocator::new());

pub const HEAP_START: usize = 0xAAA_AAA_AAA;
pub const HEAP_SIZE: usize = 100 * 1024;

// TODO! make the memory module more generic for different architectures; for now we can only support x86_64 because of the bootloader crate so take into account making our own bootloader for aarch64
pub unsafe fn init_memory(
    physical_mem_addr: &'static mut Optional<u64>,
    memory_regions: &'static mut MemoryRegions,
) -> Result<(), MapToError> {
    let phy_offset = physical_mem_addr.take().unwrap();

    let level_4_table = unsafe { page::level_4_table(phy_offset) };

    let mut mapper = Mapper::new(phy_offset as PhysAddr, level_4_table);
    //
    let page_range = {
        let heap_start = HEAP_START;
        let heap_end = heap_start + HEAP_SIZE - 1;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        println!("allocated heap pages");
        Page::iter_pages(heap_start_page, heap_end_page)
    };
    println!("{:#?}", page_range);
    //
    let frame_allocator = &mut RegionAllocator::new(memory_regions);

    let flags = EntryFlags::PRESENT | EntryFlags::WRITABLE;
    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        println!("page {:#?}", page);
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

    GLOBAL_ALLOCATOR.inner.lock().init(HEAP_START, HEAP_SIZE);
    Ok(())
}
