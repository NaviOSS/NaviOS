// a pmm i believe

use core::slice;

use crate::serial;

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

#[derive(Debug)]
pub struct RegionAllocator {
    /// keeps track of which frame is used or not
    pub bitmap: Bitmap,
    /// the index of the frame we start searching from in the bitmap
    search_from: usize,
}

impl RegionAllocator {
    /// limine
    /// TODO: look at setting unsable frames as used in the bitmap
    pub fn new() -> Self {
        let mmap = crate::limine::mmap_request();
        // figuring out how much frames we have
        let mut last_usable_entry = None;
        let mut first_usable_entry = None;

        for entry in mmap.entries() {
            if entry.entry_type == limine::memory_map::EntryType::USABLE {
                if first_usable_entry.is_none() {
                    first_usable_entry = Some(entry);
                }
                last_usable_entry = Some(entry);
            }
        }

        let last_usable_entry = last_usable_entry.unwrap();

        let frame_count = align_down(
            (last_usable_entry.base + last_usable_entry.length) as usize,
            PAGE_SIZE,
        ) / PAGE_SIZE;

        serial!("{} usable bytes found\n", frame_count * PAGE_SIZE);

        // frame_count is the number of bits
        // aligns to 8 to make sure we can get a vaild number of bytes for our frame
        let bytes = align_up(frame_count, 8) / 8;

        // finds a place the bitmap can live in
        let mut best_region: Option<&limine::memory_map::Entry> = None;

        for entry in mmap.entries() {
            if entry.entry_type == limine::memory_map::EntryType::USABLE {
                if entry.length as usize >= bytes {
                    if best_region.is_none() || best_region.is_some_and(|x| x.length > entry.length)
                    {
                        best_region = Some(entry);
                    }
                }
            }
        }

        assert!(best_region.is_some());
        serial!(
            "expected {} bytes but found a region with {} bytes\n",
            bytes,
            best_region.unwrap().length
        );

        // allocates and setups bitmap
        let bitmap_base = best_region.unwrap().base as usize;
        let bitmap_length = best_region.unwrap().length as usize;

        let addr = (bitmap_base + crate::limine::get_phy_offset()) as *mut u8;

        let bitmap = unsafe { slice::from_raw_parts_mut(addr, bytes) };
        bitmap.fill(0xFF);

        assert!(bitmap[0] == 0xFF);

        let mut this = Self {
            bitmap,
            search_from: Self::bitmap_index_from_addr(align_up(
                first_usable_entry.unwrap().base as usize,
                PAGE_SIZE,
            )),
        };

        serial!("bitmap allocation successful!\n");
        // sets all unusable frames as used
        for entry in mmap.entries() {
            if entry.entry_type == limine::memory_map::EntryType::USABLE {
                this.set_unused_from(entry.base as PhysAddr, entry.length as usize);
            }

            if entry.base == last_usable_entry.base {
                break;
            }
        }

        this.set_used_from(bitmap_base, bitmap_length);
        this
    }

    #[inline]
    fn set_used_from(&mut self, from: PhysAddr, size: usize) {
        let frames_needed = align_up(size, PAGE_SIZE) / PAGE_SIZE;

        for frame in 0..frames_needed {
            self.set_used(from + frame * PAGE_SIZE);
        }
    }

    #[inline]
    fn set_unused_from(&mut self, from: PhysAddr, size: usize) {
        let frames_needed = align_down(size, PAGE_SIZE) / PAGE_SIZE;

        for frame in 0..frames_needed {
            self.set_unused(from + frame * PAGE_SIZE);
        }
    }

    /// takes a bitmap index(bitnumber) and turns it into (row, col)
    #[inline]
    fn bitmap_loc_from_index(index: usize) -> (usize, usize) {
        (index / 8, index % 8)
    }

    /// takes an addr and turns it into a bitmap (row, col)
    #[inline]
    fn bitmap_loc_from_addr(addr: PhysAddr) -> (usize, usize) {
        Self::bitmap_loc_from_index(align_down(addr, PAGE_SIZE) / PAGE_SIZE)
    }

    #[inline]
    fn bitmap_index_from_addr(addr: PhysAddr) -> usize {
        let (row, col) = Self::bitmap_loc_from_addr(addr);
        Self::bitmap_index_from_loc(row, col)
    }

    /// returns the bitmap index of row, col aka bitnumber
    #[inline]
    fn bitmap_index_from_loc(row: usize, col: usize) -> usize {
        row * 8 + col
    }

    #[inline]
    fn search_for_free_frame(&mut self) -> Option<Frame> {
        let (srow, _) = Self::bitmap_loc_from_index(self.search_from);

        for row in srow..self.bitmap.len() {
            for col in 0..8 {
                if (self.bitmap[row] >> col) & 1 == 0 {
                    return Some(Frame {
                        start_address: Self::bitmap_index_from_loc(row, col) * PAGE_SIZE,
                    });
                }
            }
        }

        None
    }

    pub fn allocate_frame(&mut self) -> Option<Frame> {
        let frame = self.search_for_free_frame()?;
        self.set_used(frame.start_address);

        Some(frame)
    }

    fn set_unused(&mut self, addr: PhysAddr) {
        let (row, col) = Self::bitmap_loc_from_addr(addr);
        self.bitmap[row] = self.bitmap[row] ^ (1 << col)
    }

    fn set_used(&mut self, addr: PhysAddr) {
        let (row, col) = Self::bitmap_loc_from_addr(addr);
        self.bitmap[row] = self.bitmap[row] | (1 << col)
    }

    pub fn deallocate_frame(&mut self, frame: Frame) {
        self.set_unused(frame.start_address);
    }
}
