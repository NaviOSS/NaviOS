// TODO: configure this into a bitmap or anything different such as a linked list to allow
// deallocating
// a pmm i believe

use bootloader_api::info::{MemoryRegionKind, MemoryRegions};

use super::{align_down, PhysAddr};
#[derive(Debug, Clone, Copy)]
pub struct Frame {
    pub start_address: PhysAddr,
}

impl Frame {
    #[inline]
    // returns the frame that contains an address
    pub fn containing_address(address: PhysAddr) -> Self {
        Self {
            start_address: align_down(address, 4096), // for now frames can only be 1 normal page sized
        }
    }
}

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

    fn usable_regions(&self) -> impl Iterator<Item = Frame> + '_ {
        let usable_regions = self
            .memory_map
            .iter()
            .filter(|x| x.kind == MemoryRegionKind::Usable);
        let addr_ranges = usable_regions.map(|x| x.start..x.end);

        let address = addr_ranges.flat_map(|x| x.step_by(4096));

        address.map(|x| Frame::containing_address(x as PhysAddr))
    }
}

impl RegionAllocator {
    pub fn allocate_frame(&mut self) -> Option<Frame> {
        let region = self.usable_regions().nth(self.next);
        self.next += 1;
        region
    }

    pub fn deallocate_frame(&mut self, frame: Frame) {}
}
