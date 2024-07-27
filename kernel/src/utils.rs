// TODO: impl our own Optional type
pub use bootloader_api::info::Optional;

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
