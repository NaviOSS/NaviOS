pub mod allocator;
pub mod frame_allocator;

use bootloader_api::info::{MemoryRegions, Optional};
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, OffsetPageTable, Page, PageTable,
        PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

use crate::utils::Locked;

use self::frame_allocator::RegionAllocator;
pub fn align(addr: usize, align: usize) -> usize {
    let remainder = addr % align;
    if remainder == 0 {
        addr
    } else {
        addr - remainder + align
    }
}

#[global_allocator]
pub static GLOBAL_ALLOCATOR: Locked<allocator::LinkedListAllocator> =
    Locked::new(allocator::LinkedListAllocator::new());

#[cfg(target_arch = "x86_64")]
pub unsafe fn level_4_table(phy_offset: u64) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;
    let physical_addr = Cr3::read().0;
    let virt_addr = physical_addr.start_address().as_u64() + phy_offset;

    &mut *(virt_addr as *mut PageTable)
}

pub const HEAP_START: usize = 0x_ADD_E000;
pub const HEAP_SIZE: usize = 100 * 1024;

// TODO! make the memory module more generic for different architectures; for now we can only support x86_64 because of the bootloader crate so take into account making our own bootloader for aarch64
pub unsafe fn init_memory(
    physical_mem_addr: &'static mut Optional<u64>,
    memory_regions: &'static mut MemoryRegions,
) -> Result<OffsetPageTable<'static>, MapToError<Size4KiB>> {
    let phy_offset = physical_mem_addr.take().unwrap();

    let level_4_table = unsafe { level_4_table(phy_offset) };

    let mut mapper = OffsetPageTable::new(level_4_table, VirtAddr::new(phy_offset));

    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE as u64 - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    let frame_allocator = &mut RegionAllocator::new(memory_regions);

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

    GLOBAL_ALLOCATOR.inner.lock().init(HEAP_START, HEAP_SIZE);
    Ok(mapper)
}
