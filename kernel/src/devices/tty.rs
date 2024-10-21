use core::fmt::Write;

use alloc::string::String;
use spin::RwLock;

use crate::{terminal::TTY, threading::expose::thread_yeild};

use super::CharDevice;

impl CharDevice for RwLock<TTY<'_>> {
    fn name(&self) -> &'static str {
        "tty"
    }

    fn read(&self, buffer: &mut [u8]) -> usize {
        self.write().enable_input();
        while !self.read().stdin_buffer.ends_with('\n') {
            thread_yeild();
        }
        self.write().disable_input();

        if self.read().stdin_buffer.len() <= buffer.len() {
            let count = self.read().stdin_buffer.len();
            buffer[..count].copy_from_slice(self.read().stdin_buffer.as_bytes());
            self.write().stdin_buffer.clear();
            count
        } else {
            let count = buffer.len();
            buffer[..count].copy_from_slice(&self.read().stdin_buffer.as_bytes()[..count]);
            self.write().stdin_buffer.drain(..count);
            count
        }
    }

    fn write(&self, buffer: &[u8]) -> usize {
        let _ = self.write().write_str(&String::from_utf8_lossy(buffer));
        buffer.len()
    }
}
