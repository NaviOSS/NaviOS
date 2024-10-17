//! bump allocator for large kernel allocations

use core::{
    alloc::{AllocError, Allocator, GlobalAlloc},
    mem::MaybeUninit,
};

use crate::{debug, kernel, utils::Locked};

use super::{
    align_up,
    paging::{current_root_table, EntryFlags, IterPage, Page, PAGE_SIZE},
    sorcery::ROOT_BINDINGS,
};

/// Allocator for large kernel memory allocations.
pub struct PageAllocator {
    heap_start: usize,
    heap_end: usize,
    current_page: Page,
    allocations: usize,
}

impl PageAllocator {
    pub fn init(&mut self) {
        let (start, size) = ROOT_BINDINGS
            .get("LARGE_HEAP")
            .expect("failed to get LARGE_HEAP binding");
        debug!(PageAllocator, "initialized allocator");
        self.heap_start = start as usize;
        self.heap_end = self.heap_start + size;
        self.current_page = Page::containing_address(self.heap_start);
        self.allocations = 0;
    }
    #[inline(always)]
    pub fn bump(&mut self) -> Option<Page> {
        if self.current_page.start_address >= self.heap_end {
            return None;
        }

        let results = self.current_page;
        self.current_page = Page::containing_address(self.current_page.start_address + PAGE_SIZE);
        Some(results)
    }

    pub fn shrink(&mut self) {
        self.current_page = Page::containing_address(self.current_page.start_address - PAGE_SIZE);
        unsafe {
            current_root_table().unmap(self.current_page);
        }
    }

    /// allocates `page_count` number of contiguous pages
    /// returns a pointer to the start of the allocated memory, or an error if allocation fails.
    pub fn allocmut(&mut self, page_count: usize) -> Option<*mut u8> {
        let mut results = None;

        if page_count == 1 {
            let start_page = Page::containing_address(self.heap_start);
            let end_page = Page::containing_address(self.heap_end);

            let mut iter = IterPage {
                start: start_page,
                end: end_page,
            };

            while let Some(page) = iter.next() {
                unsafe {
                    if current_root_table().get_frame(page).is_none() {
                        current_root_table()
                            .map_to(
                                page,
                                kernel().frame_allocator().allocate_frame()?,
                                EntryFlags::PRESENT | EntryFlags::WRITABLE,
                            )
                            .ok()?;

                        results = Some(page.start_address as *mut u8);
                    }
                }
            }
        } else {
            let page = self.bump()?;
            unsafe {
                let frame = kernel().frame_allocator().allocate_frame()?;
                current_root_table()
                    .map_to(page, frame, EntryFlags::PRESENT | EntryFlags::WRITABLE)
                    .ok()?;

                results = Some(page.start_address as *mut u8);
            }

            for i in 1..page_count {
                let page = self.bump();

                if page.is_none() {
                    for _ in 0..i + 1 {
                        self.shrink();
                        return None;
                    }
                }
                unsafe {
                    let frame = kernel().frame_allocator().allocate_frame()?;
                    current_root_table()
                        .map_to(
                            page.unwrap_unchecked(),
                            frame,
                            EntryFlags::PRESENT | EntryFlags::WRITABLE,
                        )
                        .ok()?;
                }
            }
        }
        if results.is_some() {
            self.allocations += 1;
        }
        results
    }

    unsafe fn deallocmut(&mut self, ptr: *mut u8, size: usize) {
        let start = Page::containing_address(ptr as usize);
        let end = Page::containing_address(start.start_address + size);

        let iter = IterPage { start, end };

        for page in iter {
            if page == Page::containing_address(self.current_page.start_address - PAGE_SIZE) {
                self.shrink();
                continue;
            }

            unsafe {
                current_root_table().unmap(page);
            }
        }

        self.allocations -= 1;
        if self.allocations == 0 {
            self.current_page = Page::containing_address(self.heap_start);
        }
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
