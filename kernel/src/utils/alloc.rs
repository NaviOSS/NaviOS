use core::ops::RangeBounds;
use core::str;

use crate::memory::page_allocator::{PageAlloc, GLOBAL_PAGE_ALLOCATOR};
use crate::memory::{align_up, paging::PAGE_SIZE};
use alloc::str::pattern::{Pattern, ReverseSearcher};
use alloc::vec::{Drain, Vec};

pub struct PageVec<T> {
    inner: Vec<T, PageAlloc>,
}

impl<T> PageVec<T> {
    pub fn new() -> Self {
        Self {
            inner: Vec::new_in(&*GLOBAL_PAGE_ALLOCATOR),
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn reserve(&mut self, additional: usize) {
        let additional = align_up(additional, PAGE_SIZE / core::mem::size_of::<T>());
        self.inner.reserve(additional);
    }

    pub fn extend_from_slice(&mut self, other: &[T])
    where
        T: Clone,
    {
        if self.inner.capacity() == self.inner.len() {
            self.reserve(other.len());
        }
        self.inner.extend_from_slice(other);
    }

    pub fn truncate(&mut self, len: usize) {
        self.inner.truncate(len);
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn drain<R: RangeBounds<usize>>(&mut self, range: R) -> Drain<'_, T, PageAlloc> {
        self.inner.drain(range)
    }
}

impl<T> core::ops::Deref for PageVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct PageString {
    pub inner: PageVec<u8>,
}

impl PageString {
    pub fn new() -> Self {
        Self {
            inner: PageVec::new(),
        }
    }

    #[inline]
    pub fn push_str(&mut self, s: &str) {
        self.inner.extend_from_slice(s.as_bytes());
    }

    pub fn push_char(&mut self, c: char) {
        let mut dst = [0; 4];
        let fake_str = c.encode_utf8(&mut dst);
        self.push_str(fake_str);
    }

    pub fn pop(&mut self) -> Option<char> {
        let char = self.as_str().chars().rev().next()?;
        self.inner.truncate(self.len() - char.len_utf8());
        Some(char)
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.inner) }
    }

    pub fn ends_with<P>(&self, other: P) -> bool
    where
        P: Pattern,
        for<'a> P::Searcher<'a>: ReverseSearcher<'a>,
    {
        self.as_str().ends_with(other)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}
