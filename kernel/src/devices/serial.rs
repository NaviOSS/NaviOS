use core::fmt::Write;

use crate::{
    arch::serial::Serial,
    drivers::vfs::{FSError, FSResult},
    utils::Locked,
};

use super::CharDevice;

impl CharDevice for Locked<Serial> {
    fn name(&self) -> &'static str {
        "ss"
    }

    fn read(&self, _buffer: &mut [u8]) -> FSResult<usize> {
        FSResult::Err(FSError::OperationNotSupported)
    }

    fn write(&self, buffer: &[u8]) -> FSResult<usize> {
        let str = unsafe { core::str::from_utf8_unchecked(buffer) };

        self.try_lock()
            .ok_or(FSError::ResourceBusy)?
            .write_str(str)
            .unwrap();
        FSResult::Ok(buffer.len())
    }
}
