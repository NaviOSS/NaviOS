use alloc::{string::String, vec::Vec};

use core::{fmt, ptr};

use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use noto_sans_mono_bitmap::{FontWeight, RasterHeight, RasterizedChar};

use crate::{
    drivers::keyboard::{Key, KeyCode, KeyFlags},
    memory::align_down,
    print, println,
};

use super::{
    navitts::{Attributes, NaviTTES},
    process_command,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalMode {
    Init,
    Stdout,
    Stdin,
    Input,
}

const RASTER_HEIGHT: RasterHeight = RasterHeight::Size20;
const WRITE_COLOR: (u8, u8, u8) = (255, 255, 255);

pub struct Terminal<'a> {
    row: usize,
    column: usize,
    buffer: &'a mut [u8],

    viewport: Vec<u8>,
    viewport_start: usize,

    pub mode: TerminalMode,
    pub stdin_buffer: String,
    pub stdout_buffer: String,

    pub info: FrameBufferInfo,
    pub x_pos: usize,
    pub y_pos: usize,
}

impl<'a> Terminal<'a> {
    pub fn init(frame_buffer: &'static mut FrameBuffer) -> Self {
        let frame_buffer = frame_buffer;
        let info = frame_buffer.info();
        let buffer = frame_buffer.buffer_mut();

        for i in 0..buffer.len() {
            buffer[i] = 0;
        }

        let mut viewport = Vec::new();
        viewport.resize(buffer.len(), 0);

        Self {
            row: 0,
            column: 0,
            buffer,
            viewport,
            viewport_start: 0,

            mode: TerminalMode::Init,
            stdin_buffer: String::new(),
            stdout_buffer: String::new(),

            info,
            x_pos: 0,
            y_pos: 0,
        }
    }

    pub fn on_key_pressed(&mut self, key: Key) {
        match key.code {
            KeyCode::PageDown => self.scroll_down(false),
            KeyCode::PageUp => self.scroll_up(),
            KeyCode::KeyC => {
                if key.flags == KeyFlags::CTRL | KeyFlags::SHIFT {
                    self.clear()
                }
            }
            _ => (),
        }

        match self.mode {
            TerminalMode::Stdin => {
                let mapped = key.map_key();

                if mapped != '\0' {
                    self.stdin_putc(mapped)
                }
            }

            // input mode is just stdin however we dont have a 'program' which executes
            // process_command everytime a \n happens (like `readln()`) and we cannot do that
            // without threading
            TerminalMode::Input => {
                let mapped = key.map_key();

                if mapped == '\n' {
                    self.newline();

                    let buffer = self.stdin_buffer.clone();
                    self.stdin_buffer.clear();

                    process_command(buffer);
                    return;
                }

                if mapped != '\0' {
                    self.stdin_putc(mapped)
                }
            }
            _ => (),
        }
    }

    fn width(&self) -> usize {
        self.info.width
    }

    fn height(&self) -> usize {
        self.info.height
    }

    pub fn draw_viewport(&mut self) {
        self.buffer.copy_from_slice(
            &self.viewport[self.viewport_start..self.buffer.len() + self.viewport_start],
        );
    }

    pub fn clear(&mut self) {
        println!("clearing");
        self.viewport_start = 0;
        self.viewport.truncate(self.buffer.len());
        self.viewport.fill(0);

        self.x_pos = 0;
        self.y_pos = 0;

        self.stdin_buffer = String::new();
        self.stdout_buffer = String::new();
        if self.mode == TerminalMode::Init {
            print!(
                r"\[fg: (0, 255, 0) ||
 _   _             _  ____   _____
| \ | |           (_)/ __ \ / ____|
|  \| | __ ___   ___| |  | | (___
| . ` |/ _` \ \ / / | |  | |\___ \
| |\  | (_| |\ V /| | |__| |____) |
|_| \_|\__,_| \_/ |_|\____/|_____/
||]"
            );
            print!(
                "\\[fg: (255, 255, 255) ||\nwelcome to NaviOS!\ntype help or ? for a list of avalible commands\n||]"
            );
        }

        self.draw_viewport();
        self.enter_stdin()
    }

    fn get_byte_offset(&self, x: usize, y: usize) -> usize {
        (y * self.info.stride + x) * self.info.bytes_per_pixel
    }

    #[inline]
    fn scroll_amount(&self) -> usize {
        self.info.stride * self.info.bytes_per_pixel * RASTER_HEIGHT.val()
    }

    fn scroll_up(&mut self) {
        let scroll_amount = self.scroll_amount();
        if self.viewport_start >= scroll_amount {
            self.viewport_start -= scroll_amount;
            self.draw_viewport()
        }
    }

    /// if make_space it resizes viewport if possible if not removes the first line from buffer
    /// (shifts the buffer up by 1 line)
    fn scroll_down(&mut self, make_space: bool) {
        let scroll_amount = self.scroll_amount();

        // this should only execute if we were scrolling using page up and page down
        if !make_space
            && self.viewport.len() >= self.viewport_start + scroll_amount + self.buffer.len()
        {
            self.viewport_start += scroll_amount;
            self.draw_viewport();
        } else if make_space {
            if self.viewport.len() + scroll_amount <= self.buffer.len() * 4 {
                self.viewport.resize(self.viewport.len() + scroll_amount, 0);
                self.viewport_start += scroll_amount;

                self.draw_viewport()
            } else {
                let len = self.viewport.len();

                self.viewport.copy_within(scroll_amount..len, 0);
                self.viewport[len - scroll_amount..len].fill(0);

                self.y_pos -= RASTER_HEIGHT.val();

                self.draw_viewport();
            }
        }
    }

    fn newline(&mut self) {
        self.y_pos += RASTER_HEIGHT.val();
        self.x_pos = 0;
    }

    pub fn remove_char(&mut self, c: char) {
        let glyph = Self::raster(c);

        if self.x_pos >= glyph.width() {
            self.x_pos -= glyph.width();
        } else {
            self.y_pos -= RASTER_HEIGHT.val();
            self.x_pos = align_down(self.info.stride, glyph.width());
            self.x_pos -= glyph.width();
        }

        for (row, rows) in glyph.raster().iter().enumerate() {
            for (col, _) in rows.iter().enumerate() {
                self.set_pixel(self.x_pos + col, self.y_pos + row, 0, (0, 0, 0));
            }
        }
    }

    pub fn backspace(&mut self) {
        self.stdin_buffer.pop(); // popping backspace
        let Some(char) = self.stdin_buffer.pop() else {
            return;
        };
        self.remove_char(char)
    }

    fn set_pixel(&mut self, x: usize, y: usize, intens: u32, color: (u8, u8, u8)) {
        let color = (color.0 as u32, color.1 as u32, color.2 as u32);

        let color = match self.info.pixel_format {
            PixelFormat::Rgb => [intens * color.0, intens * color.1, intens * color.2, 0],
            PixelFormat::Bgr => [intens * color.2, intens * color.1, intens * color.0, 0],
            other => {
                panic!("pixel format {:?} not supported", other)
            }
        };
        let bytes_per_pixel = self.info.bytes_per_pixel;
        let byte_offset = self.get_byte_offset(x, y);

        // i dont know why this works just did some random stuff but it does!
        let color = [
            (color[0] >> 8 & 0xff) as u8,
            (color[1] >> 8 & 0xff) as u8,
            (color[2] >> 8 & 0xff) as u8,
            (color[3] >> 8 & 0xff) as u8,
        ];

        self.viewport[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);

        unsafe {
            ptr::read_volatile(self.buffer.as_ptr()); // ensure buffer is not optimized away
        }
    }

    fn draw_char(&mut self, glyph: RasterizedChar, color: (u8, u8, u8)) {
        if (self.x_pos + glyph.width()) > self.width() {
            self.newline();
        }

        if self.y_pos * self.info.stride * self.info.bytes_per_pixel
            >= self.viewport_start + self.buffer.len()
        {
            self.scroll_down(true);
        }

        for (row, rows) in glyph.raster().iter().enumerate() {
            for (col, byte) in rows.iter().enumerate() {
                self.set_pixel(self.x_pos + col, self.y_pos + row, *byte as u32, color);
            }
        }

        self.x_pos += glyph.width();
    }

    fn raster(c: char) -> RasterizedChar {
        let null =
            noto_sans_mono_bitmap::get_raster('N', FontWeight::Regular, RASTER_HEIGHT).unwrap();
        let glyph =
            noto_sans_mono_bitmap::get_raster(c, FontWeight::Bold, RASTER_HEIGHT).unwrap_or(null);
        glyph
    }

    pub fn putc(&mut self, c: char, color: (u8, u8, u8)) {
        match c {
            '\n' => {
                self.newline();
            }
            '\x08' => self.backspace(),

            _ => {
                self.draw_char(Self::raster(c), color);
            }
        }
    }

    const INPUT_CHAR: (u8, u8, u8) = (170, 200, 30);
    pub fn stdin_putc(&mut self, c: char) {
        self.stdin_buffer.push(c);

        // removing the _ if we are in input mode
        if self.mode == TerminalMode::Input {
            self.remove_char('_')
        }

        self.putc(c, (255, 255, 255));

        // puts it back
        if self.mode == TerminalMode::Input {
            self.putc('_', Self::INPUT_CHAR);
        }

        self.draw_viewport()
    }

    pub fn enter_stdin(&mut self) {
        print!(">> ");
        self.mode = TerminalMode::Input;
        self.putc('_', Self::INPUT_CHAR);
    }

    fn write_slice(&mut self, str: &str, attributes: Attributes) {
        let old_mode = self.mode;
        self.mode = TerminalMode::Stdout;
        self.stdout_buffer.push_str(str);

        for c in str.chars() {
            self.putc(c, attributes.fg);
        }

        self.draw_viewport();
        self.mode = old_mode
    }

    pub fn write_es(&mut self, escape_seq: NaviTTES, default_attributes: Attributes) {
        match escape_seq {
            NaviTTES::Slice(s) => self.write_slice(s, default_attributes),
            NaviTTES::OwnedSlice(s) => self.write_slice(s.as_str(), default_attributes),

            NaviTTES::NaviESS(escape_seqs) => {
                for escape_seq in escape_seqs {
                    self.write_es(escape_seq, default_attributes.clone())
                }
            }
            NaviTTES::NaviES(attributes, seq) => {
                let attributes = Attributes::from_list(&attributes, default_attributes);

                self.write_es(*seq, attributes)
            }
        }
    }
    pub fn write(&mut self, str: &str) {
        let parsed = NaviTTES::parse_str(str);
        let mut default_attributes = Attributes::default();
        default_attributes.fg = WRITE_COLOR;
        self.write_es(parsed, default_attributes)
    }
}

impl fmt::Write for Terminal<'static> {
    // i can add color escapes later on like parsing \(u8, u8, u8)str$ as coloring str into (u8, u8, u8)
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s);
        Ok(())
    }
}
