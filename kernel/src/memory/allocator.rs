use core::{
    alloc::{GlobalAlloc, Layout},
    ptr,
};

use crate::utils::Locked;

use super::align;
#[derive(Debug)]
pub struct Node {
    size: usize,
    next: Option<&'static mut Node>,
}

impl Node {
    pub const fn new(size: usize) -> Self {
        Self { size, next: None }
    }

    pub fn start(&self) -> usize {
        self as *const Self as usize
    }

    pub fn end(&self) -> usize {
        self.start() + self.size
    }

    pub fn can_hold(&self, size: usize, align_amount: usize) -> Result<usize, ()> {
        let start = align(self.start(), align_amount);
        let end = start.checked_add(size).ok_or(())?;

        if end > self.end() {
            return Err(());
        }

        let ecess_size = end - self.size;
        // if ecess is bigger than 0 then it should be able to hold a Node so we can divide our node
        if ecess_size > 0 && ecess_size < size_of::<Node>() {
            return Err(());
        }

        Ok(start)
    }
}
pub struct LinkedListAllocator {
    head: Node,
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        Self {
            head: Node {
                size: 0,
                next: None,
            },
        }
    }
    // heap_start has to be aligned
    pub unsafe fn init(&mut self, heap_start: usize, size: usize) {
        self.add_free_node(heap_start, size);
    }

    pub unsafe fn alloc_mut(&mut self, layout: Layout) -> *mut u8 {
        let (size, align) = Self::size_align(layout);

        if let Some((node, addr)) = self.find_free_node(size, align) {
            let alloc_end = addr.checked_add(size).expect("overflow");
            // divide block
            let excess_size = node.end() - alloc_end;
            if excess_size > 0 {
                self.add_free_node(alloc_end, excess_size);
            }

            addr as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    pub unsafe fn dealloc_mut(&mut self, ptr: *mut u8, layout: Layout) {
        let (size, _) = Self::size_align(layout);

        self.add_free_node(ptr as usize, size)
    }

    pub fn find_free_node(
        &mut self,
        size: usize,
        align: usize,
    ) -> Option<(&'static mut Node, usize)> {
        let mut current = &mut self.head;

        while let Some(ref mut node) = current.next {
            if let Ok(addr) = node.can_hold(size, align) {
                let node = current.next.take().unwrap();
                let next = node.next.take();
                current.next = next;

                return Some((node, addr));
            } else {
                current = current.next.as_mut().unwrap();
            }
        }

        None
    }

    pub unsafe fn add_free_node(&mut self, addr: usize, size: usize) {
        let mut node = Node::new(size);

        node.next = self.head.next.take();

        let node_ptr = addr as *mut Node;

        ptr::write(node_ptr, node);

        self.head.next = Some(&mut *node_ptr);
    }

    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(align_of::<Node>())
            .expect("adjusting alignment failed")
            .pad_to_align();

        let size = layout.size().max(size_of::<Node>());
        (size, layout.align())
    }
}

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.inner.lock();
        allocator.alloc_mut(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.inner.lock();
        allocator.dealloc_mut(ptr, layout)
    }
}
