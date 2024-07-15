pub mod allocator;
pub mod page;

use bootloader_api::info::{MemoryRegionKind, MemoryRegions};

use crate::{println, utils::Locked};
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

pub fn init_heap(regions: &'static mut MemoryRegions) {
    let mut usable_region = None;

    for region in regions.iter() {
        if region.kind == MemoryRegionKind::Usable {
            usable_region = Some(region);
            break;
        }
    }

    if let Some(region) = usable_region {
        let heap_start = region.start as usize;

        let heap_size = (region.end - region.start) as usize;

        println!("heap size: {:?}", heap_size);
        unsafe {
            GLOBAL_ALLOCATOR.inner.lock().init(heap_start, heap_size);
        }
    } else {
        panic!("No usable memory region found for the heap");
    }
}
