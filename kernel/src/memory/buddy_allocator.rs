use crate::debug;

use super::{align_up, VirtAddr};

#[derive(Debug, Clone)]
pub struct Block {
    free: bool,
    /// decreases header size
    /// we dont want more then 4gb of heap space anyways we want a few mbs
    size: u32,
}

impl Block {
    #[inline]
    /// unsafe because there may be no next block causing UB
    /// use BuddyAllocator::next instead
    pub unsafe fn next(&self) -> &mut Block {
        let end = (self as *const Self).byte_add(self.size as usize);
        &mut *end.cast_mut()
    }
}

#[derive(Debug)]
pub struct BuddyAllocator<'a> {
    head: &'a mut Block,
    heap_end: usize,
}

impl BuddyAllocator<'_> {
    pub unsafe fn init(possible_start: VirtAddr, size: u32) -> Self {
        let start = align_up(possible_start, size_of::<Block>());
        let start = align_up(start, 2);

        let diff = (start - possible_start) as u32;
        let size = size - diff - (size_of::<Block>() as u32);
        let end = start + size as usize;

        debug!(
            BuddyAllocator,
            "initing at {:#x}..{:#x} instead of {:#x} with size: {:#x}",
            start,
            end,
            possible_start,
            size
        );

        let head = &mut *(start as *mut Block);
        head.free = true;
        head.size = size;

        Self {
            head,
            heap_end: end,
        }
    }

    #[inline]
    /// safe wrapper around Block::next
    pub fn next(heap_end: usize, block: &Block) -> Option<&mut Block> {
        if (block as *const _ as usize + block.size as usize) >= heap_end {
            None
        } else {
            unsafe { Some(block.next()) }
        }
    }

    pub fn find_free_block(&mut self, size: usize) -> Option<&mut Block> {
        let heap_end = self.heap_end;
        let mut current = &mut *self.head;
        let mut best_block: Option<*mut Block> = None;

        while let Some(block) = Self::next(heap_end, current) {
            current = block;
            if !current.free {
                continue;
            }

            if current.size as usize == size {
                return Some(current);
            }

            if best_block.is_none()
                || best_block.as_ref().is_some_and(|x| unsafe {
                    (**x).size as usize > size && (**x).size > current.size
                })
            {
                best_block = Some(current as *mut _);
            }
        }

        None
    }
}
