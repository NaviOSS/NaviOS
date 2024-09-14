use alloc::string::String;

use crate::{
    drivers::vfs::{FSResult, FileDescriptor, FS},
    threading,
};

use super::framebuffer::{Terminal, TerminalMode, VIEWPORT};

impl FS for Terminal {
    fn name(&self) -> &'static str {
        "tty"
    }

    fn write(&mut self, _: &mut FileDescriptor, buffer: &[u8]) -> FSResult<()> {
        let str = String::from_utf8_lossy(buffer);
        while VIEWPORT.is_locked() {
            threading::expose::thread_yeild()
        }

        self.write(&str);

        Ok(())
    }

    fn read(&mut self, file_descriptor: &mut FileDescriptor, buffer: &mut [u8]) -> FSResult<usize> {
        if self.stdin_buffer.is_empty() {
            let old_mode = self.mode;
            self.mode = TerminalMode::Stdin;

            // FIXME: the thing is there is no locks here!?????????
            while !self.stdin_buffer.ends_with('\n') {
                threading::expose::thread_yeild()
            }
            self.mode = old_mode;
        }

        let file_size = self.stdin_buffer.len();
        let count = buffer.len();

        let count = if file_descriptor.read_pos + count > file_size {
            file_size - file_descriptor.read_pos
        } else {
            count
        };

        buffer[..count].copy_from_slice(
            self.stdin_buffer[file_descriptor.read_pos..file_descriptor.read_pos + count]
                .as_bytes(),
        );

        file_descriptor.read_pos += count;

        if file_descriptor.read_pos == self.stdin_buffer.len() {
            self.stdin_buffer.clear();
            file_descriptor.read_pos = 0;
        }

        Ok(count)
    }
}
