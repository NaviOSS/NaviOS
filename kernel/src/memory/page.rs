const ENTRY_COUNT: usize = 512;
const PAGE_SIZE: usize = 4096;

use crate::{memory::PhysAddr, println};
use bitflags::bitflags;
use core::{
    arch::asm,
    ops::{Index, IndexMut},
};

use crate::memory::frame_allocator::Frame;

use super::{align_down, frame_allocator::RegionAllocator, VirtAddr};

#[derive(Debug, Clone, Copy)]
pub struct Page {
    pub start_address: VirtAddr,
}
#[derive(Debug)]
pub struct IterPage {
    pub start: Page,
    pub end: Page,
}

impl Page {
    pub const fn containing_address(address: VirtAddr) -> Self {
        Self {
            start_address: align_down(address, PAGE_SIZE),
        }
    }

    pub const fn iter_pages(start: Page, end: Page) -> IterPage {
        IterPage { start, end }
    }
}

impl Iterator for IterPage {
    type Item = Page;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start.start_address <= self.end.start_address {
            let page = self.start;

            let max_page_addr = usize::MAX - (PAGE_SIZE - 1);
            if self.start.start_address < max_page_addr {
                self.start.start_address += PAGE_SIZE;
            } else {
                self.end.start_address -= PAGE_SIZE;
            }
            Some(page)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Entry(PhysAddr);
// address of the next table or physial frame in 0x000FFFFF_FFFFF000 (the fs is the address are the fs the rest are flags or reserved)

#[cfg(target_arch = "x86_64")]
impl Entry {
    pub fn frame(&self) -> Option<Frame> {
        if self.flags().contains(EntryFlags::PRESENT) {
            return Some(Frame::containing_address(self.0 & 0x000FFFFF_FFFFF000));
        }
        None
    }

    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0 as u64)
    }
}

#[cfg(target_arch = "x86_64")]
bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct EntryFlags: u64 {
        const PRESENT =         1;
        const WRITABLE =        1 << 1;
        const USER_ACCESSIBLE = 1 << 2;
        const WRITE_THROUGH =   1 << 3;
        const NO_CACHE =        1 << 4;
        const ACCESSED =        1 << 5;
        const DIRTY =           1 << 6;
        const HUGE_PAGE =       1 << 7;
        const GLOBAL =          1 << 8;
        const NO_EXECUTE =      1 << 63;
    }
}

#[derive(Debug, Clone)]
pub struct PageTable {
    entries: [Entry; ENTRY_COUNT],
}

impl Index<usize> for PageTable {
    type Output = Entry;
    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

#[cfg(target_arch = "x86_64")]
pub unsafe fn level_4_table(phy_offset: u64) -> &'static mut PageTable {
    let phys_addr: PhysAddr;
    unsafe {
        asm!("mov {}, cr3", out(reg) phys_addr);
    }
    let frame = Frame::containing_address(phys_addr);

    let virt_addr = frame.start_address + phy_offset as usize;

    &mut *(virt_addr as *mut PageTable)
}

pub enum MapToError {
    FrameAllocationFailed,
}

pub struct Mapper {
    level_4_table: &'static mut PageTable,
    offset: PhysAddr,
}

impl Mapper {
    pub const fn new(offset: PhysAddr, level_4_table: &'static mut PageTable) -> Self {
        Self {
            level_4_table,
            offset,
        }
    }

    #[cfg(target_arch = "x86_64")]
    fn map_page_table_entry(
        offset: PhysAddr,
        entry: &mut Entry,
        frame_allocator: &mut RegionAllocator,
    ) -> Result<&'static mut PageTable, MapToError> {
        use crate::println;

        if entry.flags().contains(EntryFlags::PRESENT) {
            let entry_ptr = (entry.frame().unwrap().start_address + offset) as *mut PageTable;
            println!("entry {:#?}", entry.flags());
            Ok(unsafe { &mut *(entry_ptr) })
        } else {
            let frame = frame_allocator
                .allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;
            println!("{:#?}", frame);
            let addr = frame.start_address + offset;
            entry.0 = addr | EntryFlags::PRESENT.bits() as usize;
            Ok(unsafe { &mut *(addr as *mut PageTable) })
        }
    }

    pub fn map_to(
        &mut self,
        page: Page,
        frame: Frame,
        flags: EntryFlags,
        frame_allocator: &mut RegionAllocator,
    ) -> Result<&mut Mapper, MapToError> {
        let level_4_index = (page.start_address >> 39) & 0x1FF;
        let level_3_index = (page.start_address >> 30) & 0x1FF;
        let level_2_index = (page.start_address >> 21) & 0x1FF;
        let level_1_index = (page.start_address >> 12) & 0x1FF;

        let level_4_entry = &mut self.level_4_table[level_4_index];
        let level_3_table =
            Self::map_page_table_entry(self.offset, level_4_entry, frame_allocator)?;
        println!("level 3");
        let level_2_table = Self::map_page_table_entry(
            self.offset,
            &mut level_3_table[level_3_index],
            frame_allocator,
        )?;
        println!("level 2");
        let level_1_table = Self::map_page_table_entry(
            self.offset,
            &mut level_2_table[level_2_index],
            frame_allocator,
        )?;
        println!("level 1");

        level_1_table[level_1_index] = Entry(frame.start_address | flags.bits() as usize);

        Ok(self)
    }

    pub unsafe fn flush(&self) {
        #[cfg(target_arch = "x86_64")]
        asm!("invlpg [{}]", in(reg) 0 as *const u8);
    }
}
