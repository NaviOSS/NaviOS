//! bump allocator for large kernel allocations

use core::{
    alloc::{AllocError, Allocator, GlobalAlloc},
    mem::MaybeUninit,
};

use alloc::collections::linked_list::LinkedList;

use crate::{debug, kernel, utils::Locked};

use super::{
    align_up,
    paging::{current_root_table, EntryFlags, IterPage, MapToError, Page, PAGE_SIZE},
    sorcery::ROOT_BINDINGS,
};
#[derive(Debug, Clone, PartialEq, Eq)]
struct MemoryMapping {
    start: usize,
    end: usize,
}
// TODO: make own LinkedList type because this is abanoded by rust
pub struct PageAllocator {
    heap_start: usize,
    heap_end: usize,
    mappings: LinkedList<MemoryMapping>,
}

impl PageAllocator {
    pub fn init(&mut self) {
        let (start, size) = ROOT_BINDINGS
            .get("LARGE_HEAP")
            .expect("failed to get LARGE_HEAP binding");
        debug!(PageAllocator, "initialized allocator");
        self.heap_start = start as usize;
        self.heap_end = self.heap_start + size;
    }

    /// allocates `page_count` number of contiguous pages
    /// returns a pointer to the start of the allocated memory, or an error if allocation fails.
    pub fn allocmut(&mut self, page_count: usize) -> Result<*mut u8, MapToError> {
        let start = self
            .mappings
            .back()
            .map(|mapping| mapping.end)
            .unwrap_or(self.heap_start);

        let end = start + page_count * PAGE_SIZE;
        let start_page = Page::containing_address(start);

        let iter = IterPage {
            start: start_page,
            end: Page::containing_address(end),
        };

        for page in iter {
            let frame = kernel()
                .frame_allocator()
                .allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;
            unsafe {
                current_root_table().map_to(
                    page,
                    frame,
                    EntryFlags::PRESENT | EntryFlags::WRITABLE,
                )?;
            }
        }

        self.mappings.push_back(MemoryMapping { start, end });

        Ok(start_page.start_address as *mut u8)
    }

    unsafe fn deallocmut(&mut self, ptr: *mut u8, size: usize) {
        let start = ptr as usize;
        let end = start + size;
        let this_mappings = MemoryMapping { start, end };
        for (i, mappings) in self.mappings.iter().enumerate() {
            if *mappings == this_mappings {
                let start = Page::containing_address(start);
                let end = Page::containing_address(end);

                let iter = IterPage { start, end };
                for page in iter {
                    unsafe {
                        current_root_table().unmap(page);
                    }
                }
                self.mappings.remove(i);
                return;
            }
        }

        panic!("PageAllocator: couldn't dealloc {:#x}!", ptr as usize);
    }
}

unsafe impl GlobalAlloc for Locked<PageAllocator> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.inner
            .lock()
            .allocmut((layout.size() + PAGE_SIZE - 1) / PAGE_SIZE)
            .unwrap_or(core::ptr::null_mut())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        self.inner.lock().deallocmut(ptr, layout.size())
    }
}

unsafe impl Allocator for Locked<PageAllocator> {
    fn allocate(
        &self,
        layout: core::alloc::Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, core::alloc::AllocError> {
        unsafe {
            let ptr = self.alloc(layout);
            if ptr.is_null() {
                return Err(AllocError);
            }

            let length = align_up(layout.size(), PAGE_SIZE);

            let slice = core::ptr::slice_from_raw_parts_mut(ptr, length);
            Ok(core::ptr::NonNull::new(slice).unwrap_unchecked())
        }
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        self.dealloc(ptr.as_ptr(), layout);
    }
}
pub static GLOBAL_PAGE_ALLOCATOR: MaybeUninit<Locked<PageAllocator>> = MaybeUninit::zeroed();
