use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use x86_64::{
    structures::paging::{FrameAllocator, PhysFrame, Size4KiB},
    PhysAddr,
};

pub struct RegionAllocator {
    memory_map: &'static mut MemoryRegions,
    next: usize,
}

impl RegionAllocator {
    pub fn new(memory_map: &'static mut MemoryRegions) -> Self {
        Self {
            memory_map,
            next: 0,
        }
    }

    fn usable_regions(&self) -> impl Iterator<Item = PhysFrame> + '_ {
        let usable_regions = self
            .memory_map
            .iter()
            .filter(|x| x.kind == MemoryRegionKind::Usable);
        let addr_ranges = usable_regions.map(|x| x.start..x.end);

        let address = addr_ranges.flat_map(|x| x.step_by(4096));

        address.map(|x| PhysFrame::containing_address(PhysAddr::new(x)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for RegionAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let region = self.usable_regions().nth(self.next);
        self.next += 1;
        region
    }
}
