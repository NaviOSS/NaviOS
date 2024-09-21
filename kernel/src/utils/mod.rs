pub mod elf;
pub mod expose;
pub mod ustar;

// TODO: impl our own Optional type
use spin::Mutex;

pub struct Locked<T> {
    pub inner: Mutex<T>,
}

impl<T> Locked<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            inner: Mutex::new(inner),
        }
    }
}
