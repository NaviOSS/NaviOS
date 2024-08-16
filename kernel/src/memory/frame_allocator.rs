// a pmm i believe

use alloc::slice;
use bootloader_api::info::{MemoryRegionKind, MemoryRegions};

use crate::phy_offset;

use super::{align_down, align_up, paging::PAGE_SIZE, PhysAddr};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame {
    pub start_address: PhysAddr,
}

impl Frame {
    #[inline]
    // returns the frame that contains an address
    pub fn containing_address(address: PhysAddr) -> Self {
        Self {
            start_address: align_down(address, PAGE_SIZE), // for now frames can only be 1 normal page sized
        }
    }
}

pub type Bitmap = &'static mut [u8];

fn usable_frames(mmap: &MemoryRegions) -> impl Iterator<Item = Frame> + '_ {
    let usable_regions = mmap.iter().filter(|x| x.kind == MemoryRegionKind::Usable);
    let addr_ranges = usable_regions.map(|x| x.start..x.end);

    let addr_ranges = addr_ranges.filter(|x| *x != (0..0));

    let address = addr_ranges.flat_map(|x| x.step_by(PAGE_SIZE));

    let frames = address.map(|x| Frame::containing_address(x as PhysAddr));
    frames
}

pub struct RegionAllocator {
    memory_map: &'static mut MemoryRegions,
    /// keeps track of which frame is used or not
    bitmap: Bitmap,
    /// the index of the frame we start searching from in the bitmap
    search_from: usize,
}

impl RegionAllocator {
    pub fn new(memory_map: &'static mut MemoryRegions) -> Self {
        let frame_count = usable_frames(memory_map).count();

        // frame_count is the number of bits
        // aligns to 8 to make sure we can get a vaild number of bytes for our frame
        align_up(frame_count, 8);

        let bytes = frame_count / 8;

        // finds a place the bitmap can live in
        let mut usable_regions = memory_map
            .iter_mut()
            .filter(|x| x.kind == MemoryRegionKind::Usable);

        let mut most_stable_region = None;

        for region in &mut usable_regions {
            if (region.end - region.start) as usize == bytes {
                most_stable_region = Some(region);
                break;
            }

            if (region.end - region.start) as usize > bytes {
                if most_stable_region
                    .as_ref()
                    .is_some_and(|x| (x.end - x.start) > region.end - region.start)
                {
                    most_stable_region = Some(region);
                } else if most_stable_region.is_none() {
                    most_stable_region = Some(region);
                }
            }
        }

        assert!(most_stable_region.is_some());

        let bitmap_addr: PhysAddr = most_stable_region.as_ref().unwrap().start as PhysAddr;
        let bitmap_len =
            most_stable_region.as_ref().unwrap().end - most_stable_region.as_ref().unwrap().start;

        most_stable_region.as_mut().unwrap().start = 0;
        most_stable_region.as_mut().unwrap().end = 0;

        let bitmap_ptr = (bitmap_addr + phy_offset()) as *mut u8;

        Self {
            memory_map,
            bitmap: unsafe { slice::from_raw_parts_mut(bitmap_ptr, bitmap_len as usize) },
            search_from: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = Frame> + '_ {
        usable_frames(self.memory_map)
    }

    fn bitmap_index(index: usize) -> (usize, usize) {
        (index / 8, index % 8)
    }

    fn index_bitmap(row: usize, col: usize) -> usize {
        row * 8 + col
    }

    pub fn search_for_frame(&mut self) -> Option<usize> {
        let (srow, mut scol) = Self::bitmap_index(self.search_from);

        for row in srow..self.bitmap.len() {
            if !(row == srow) {
                scol = 0;
            }

            for col in scol..8 {
                if (self.bitmap[row] >> col) & 1 == 0 {
                    return Some(Self::index_bitmap(row, col));
                }
            }
        }

        None
    }

    pub fn allocate_frame(&mut self) -> Option<Frame> {
        let index = self.search_for_frame().unwrap();

        let region = self.usable_frames().nth(index);
        self.set_used(index);
        self.search_from = index;
        region
    }

    fn set_unused(&mut self, index: usize) {
        let (row, col) = Self::bitmap_index(index);
        self.bitmap[row] = self.bitmap[row] ^ (1 << col)
    }

    fn set_used(&mut self, index: usize) {
        let (row, col) = Self::bitmap_index(index);
        self.bitmap[row] = self.bitmap[row] | (1 << col)
    }

    pub fn deallocate_frame(&mut self, frame: Frame) {
        let mut index = None;

        for (i, usable_frame) in self.usable_frames().enumerate() {
            if usable_frame == frame {
                index = Some(i);
                break;
            }
        }

        assert!(index.is_some());

        self.set_unused(index.unwrap());
        self.search_from = index.unwrap();
    }
}
