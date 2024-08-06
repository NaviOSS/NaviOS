const ENTRY_COUNT: usize = 512;
pub const PAGE_SIZE: usize = 4096;
const BITMAP_BITS_PER_ROW: usize = 64;
const ROWS: usize = 16313;
static mut BITMAP: [u64; ROWS] = [0; ROWS];

#[inline]
fn bitmap_get_bitnumber(row: usize, col: usize) -> usize {
    (row * BITMAP_BITS_PER_ROW) + col
}

#[inline]
fn bitmap_get_address(bitnumber: usize) -> usize {
    bitnumber * PAGE_SIZE
}

#[inline]
fn bitmap_get_location(address: usize) -> (usize, usize) {
    let bitnumber = address / PAGE_SIZE;
    (
        bitnumber / BITMAP_BITS_PER_ROW,
        bitnumber % BITMAP_BITS_PER_ROW,
    )
}

pub fn set_used(row: usize, col: usize) {
    unsafe { BITMAP[row] &= 1 << col }
}

pub fn set_used_addr(address: usize) {
    let (row, col) = bitmap_get_location(address);
    set_used(row, col);
}

/// fetches a free page starting from address
pub fn get_free_page_from(address: usize) -> Option<Page> {
    let address = if address != 0 {
        align_up(address, PAGE_SIZE)
    } else {
        address
    };

    let (c_row, c_col) = bitmap_get_location(address);
    for row in c_row..ROWS {
        let col = if row == c_row { c_col } else { 0 };

        for col in col..BITMAP_BITS_PER_ROW {
            let bitnumber = bitmap_get_bitnumber(row, col);

            if unsafe { BITMAP[row] as usize & 1 << col == 0 } {
                set_used(row, col);
                return Some(Page::containing_address(bitmap_get_address(bitnumber)));
            }
        }
    }

    None
}
/// fetches a free Page and marks it as used returns None if no free pages avalible
pub fn get_free_page() -> Option<Page> {
    get_free_page_from(0)
}

use crate::{
    globals::frame_allocator,
    memory::{translate, PhysAddr},
};
use bitflags::bitflags;
use core::{
    arch::asm,
    ops::{Index, IndexMut},
};

use crate::memory::frame_allocator::Frame;

use super::{align_down, align_up, frame_allocator::RegionAllocator, VirtAddr};

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

    pub const fn new(flags: EntryFlags, addr: PhysAddr) -> Self {
        Self(addr | flags.bits() as usize)
    }

    pub const fn set(&mut self, flags: EntryFlags, addr: PhysAddr) {
        *self = Self::new(flags, addr)
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

impl PageTable {
    pub fn zeroize(&mut self) {
        for entry in &mut self.entries {
            entry.0 = 0;
        }
    }
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
#[derive(Debug)]
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
        flags: EntryFlags,
        entry: &mut Entry,
        frame_allocator: &mut RegionAllocator,
    ) -> Result<&'static mut PageTable, MapToError> {
        if entry.flags().contains(EntryFlags::PRESENT) {
            let addr = entry.frame().unwrap().start_address;

            entry.set(flags | entry.flags(), addr);
            let virt_addr = addr + offset;
            let entry_ptr = virt_addr as *mut PageTable;

            Ok(unsafe { &mut *(entry_ptr) })
        } else {
            let frame = frame_allocator
                .allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;

            let addr = frame.start_address;
            entry.set(flags, addr);

            let virt_addr = addr + offset;
            let table_ptr = virt_addr as *mut PageTable;

            Ok(unsafe {
                (*table_ptr).zeroize();
                &mut *(table_ptr)
            })
        }
    }

    pub fn map_to(
        &mut self,
        page: Page,
        frame: Frame,
        flags: EntryFlags,
    ) -> Result<&mut Mapper, MapToError> {
        let (_, level_1_index, level_2_index, level_3_index, level_4_index) =
            translate(page.start_address);
        let frame_allocator = frame_allocator();

        let level_4_entry = &mut self.level_4_table[level_4_index];
        let level_3_table =
            Self::map_page_table_entry(self.offset, flags, level_4_entry, frame_allocator)?;

        let level_2_table = Self::map_page_table_entry(
            self.offset,
            flags,
            &mut level_3_table[level_3_index],
            frame_allocator,
        )?;

        let level_1_table = Self::map_page_table_entry(
            self.offset,
            flags,
            &mut level_2_table[level_2_index],
            frame_allocator,
        )?;

        let entry = &mut level_1_table[level_1_index];

        *entry = Entry::new(flags, frame.start_address);
        /*         set_used_addr(page.start_address); */
        Ok(self)
    }

    pub unsafe fn flush(&self) {
        #[cfg(target_arch = "x86_64")]
        asm!("invlpg [{}]", in(reg) 0 as *const u8);
    }

    /// maps a page starting from an address
    pub fn map_free_page_from(
        &mut self,
        address: usize,
        flags: EntryFlags,
    ) -> Result<(Page, &mut Self), MapToError> {
        let allocator = frame_allocator();

        let frame = allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        let page = get_free_page_from(address).ok_or(MapToError::FrameAllocationFailed)?;
        self.map_to(page, frame, flags | EntryFlags::PRESENT)?;

        Ok((page, self))
    }

    /// maps a free page, wrapper around map_free_page_from starting from 0
    pub fn map_free_page(&mut self, flags: EntryFlags) -> Result<(Page, &mut Self), MapToError> {
        self.map_free_page_from(0, flags)
    }
}
