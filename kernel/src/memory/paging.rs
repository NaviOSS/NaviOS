const ENTRY_COUNT: usize = 512;
pub const PAGE_SIZE: usize = 4096;
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
    pub entries: [Entry; ENTRY_COUNT],
}

impl PageTable {
    pub fn zeroize(&mut self) {
        for entry in &mut self.entries {
            entry.0 = 0;
        }
    }

    /// copies the higher half entries of the current pml4 to this page table
    pub fn copy_higher_half(&mut self, phy_offset: VirtAddr) {
        unsafe {
            self.entries[256..ENTRY_COUNT]
                .clone_from_slice(&level_4_table(phy_offset).entries[256..ENTRY_COUNT])
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
pub unsafe fn level_4_table(phy_offset: VirtAddr) -> &'static mut PageTable {
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
    phy_offset: PhysAddr,
}

impl Mapper {
    pub const fn new(phy_offset: PhysAddr) -> Self {
        Self { phy_offset }
    }

    #[cfg(target_arch = "x86_64")]
    fn map_page_table_entry(
        phy_offset: VirtAddr,
        flags: EntryFlags,
        entry: &mut Entry,
        frame_allocator: &mut RegionAllocator,
    ) -> Result<&'static mut PageTable, MapToError> {
        if entry.flags().contains(EntryFlags::PRESENT) {
            let addr = entry.frame().unwrap().start_address;

            entry.set(flags | entry.flags(), addr);
            let virt_addr = addr + phy_offset;
            let entry_ptr = virt_addr as *mut PageTable;

            Ok(unsafe { &mut *(entry_ptr) })
        } else {
            let frame = frame_allocator
                .allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;

            let addr = frame.start_address;
            entry.set(flags, addr);

            let virt_addr = addr + phy_offset;
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

        let level_4_table = unsafe { level_4_table(self.phy_offset) };

        let level_4_entry = &mut level_4_table[level_4_index];
        let level_3_table =
            Self::map_page_table_entry(self.phy_offset, flags, level_4_entry, frame_allocator)?;

        let level_2_table = Self::map_page_table_entry(
            self.phy_offset,
            flags,
            &mut level_3_table[level_3_index],
            frame_allocator,
        )?;

        let level_1_table = Self::map_page_table_entry(
            self.phy_offset,
            flags,
            &mut level_2_table[level_2_index],
            frame_allocator,
        )?;

        let entry = &mut level_1_table[level_1_index];

        *entry = Entry::new(flags, frame.start_address);
        Ok(self)
    }

    pub unsafe fn flush(&self) {
        #[cfg(target_arch = "x86_64")]
        asm!("invlpg [{}]", in(reg) 0 as *const u8);
    }

    /// allocates a pml4 and returns its physical address
    pub fn allocate_pml4(&self) -> Result<PhysAddr, MapToError> {
        let frame = frame_allocator()
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        let virt_start_addr = frame.start_address + self.phy_offset;
        let table = unsafe { &mut *(virt_start_addr as *mut PageTable) };

        table.zeroize();
        table.copy_higher_half(self.phy_offset);

        Ok(frame.start_address)
    }
}
