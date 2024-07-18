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

    let page_range = {
        let heap_start = HEAP_START;
        let heap_end = heap_start + HEAP_SIZE - 1;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        println!("allocated heap pages");
        Page::iter_pages(heap_start_page, heap_start_page)
    };
    println!("{:#?}", page_range);

    let start = page_range.start.clone();
    let end = page_range.end.clone();

    let frame_allocator = &mut RegionAllocator::new(memory_regions);

    let flags = EntryFlags::PRESENT | EntryFlags::WRITABLE | EntryFlags::USER_ACCESSIBLE;
    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        println!("page {:#?}", page);
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

    println!("start {:18x}", start.start_address);
    println!("end {:18x}", end.start_address);
    println!("heap_start {:18x}", HEAP_START);
    println!("heap_end {:18x}", HEAP_START + HEAP_SIZE);

    GLOBAL_ALLOCATOR.inner.lock().init(HEAP_START, HEAP_SIZE);
    Ok(())
}
