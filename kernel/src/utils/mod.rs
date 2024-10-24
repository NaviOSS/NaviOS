pub mod display;
pub mod elf;
pub mod expose;
pub mod ustar;

use core::ops::Deref;

// TODO: impl our own Optional type
use spin::Mutex;

pub struct Locked<T: ?Sized> {
    pub inner: Mutex<T>,
}

impl<T> Locked<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            inner: Mutex::new(inner),
        }
    }
}

impl<T> Deref for Locked<T> {
    type Target = Mutex<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
