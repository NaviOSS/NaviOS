/// TODO: fix the framebuffer acting like BGR even tho it is RGB
use alloc::{string::String, vec::Vec};
use lazy_static::lazy_static;
use spin::{Mutex, MutexGuard};

use core::ptr;

use noto_sans_mono_bitmap::{FontWeight, RasterHeight, RasterizedChar};

use crate::{
    debug,
    drivers::keyboard::{Key, KeyCode, KeyFlags},
    memory::align_down,
    serial, TERMINAL,
};

use super::navitts::{Attributes, NaviTTES};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalMode {
    Init,
    Stdout,
    Stdin,
}

#[derive(Debug, Clone, Copy)]
pub enum PixelFormat {
    Rgb,
    #[allow(dead_code)]
    /// TODO: use
    Bgr,
}
#[derive(Debug)]
pub struct FrameBufferInfo {
    /// number of pixels between start of a line and another
    pub stride: usize,
    pub bytes_per_pixel: usize,
    pub pixel_format: PixelFormat,
}

const RASTER_HEIGHT: RasterHeight = RasterHeight::Size20;
const WRITE_COLOR: (u8, u8, u8) = (255, 255, 255);
#[derive(Debug)]
pub struct Terminal {
    /// this is a lock indpendant of VIEWPORT lock, it is used only with Write::write_fmt
    buffer: &'static mut [u8],

    viewport_start: usize,

    pub mode: TerminalMode,

    pub stdin_buffer: String,
    pub stdout_buffer: String,

    pub info: FrameBufferInfo,
    /// x_pos in pixels
    pub x_pos: usize,
    /// y_pos in pixels
    pub y_pos: usize,
    /// wether or not the current panic happend in another panic because of the terminal
    pub panicked: bool,
}

lazy_static! {
    pub static ref VIEWPORT: Mutex<Vec<u8>> = Mutex::new(Vec::new());
}

impl Terminal {
    pub fn init() {
        debug!(Terminal, "initing (framebuffer) ...");
        let (buffer, info) = crate::limine::get_framebuffer();

        for i in 0..buffer.len() {
            buffer[i] = 0;
        }

        serial!("{} {:#?}\n", buffer.len(), info);

        VIEWPORT.lock().resize(buffer.len(), 0);
        let this = Self {
            buffer,
            viewport_start: 0,

            mode: TerminalMode::Init,
            stdin_buffer: String::new(),
            stdout_buffer: String::new(),

            info,
            x_pos: 0,
            y_pos: 0,
            panicked: false,
        };

        unsafe { TERMINAL = Some(this) };
        debug!(Terminal, "done ...");
    }

    pub fn on_key_pressed(&mut self, key: Key) {
        match key.code {
            KeyCode::PageDown => self.scroll_down(false, &mut VIEWPORT.lock()),
            KeyCode::PageUp => self.scroll_up(&mut VIEWPORT.lock()),
            KeyCode::KeyC => {
                if key.flags == KeyFlags::CTRL | KeyFlags::SHIFT {
                    return self.clear(&mut VIEWPORT.lock());
                }
            }
            _ => (),
        }

        match self.mode {
            TerminalMode::Stdin => {
                let mapped = key.map_key();

                if mapped != '\0' {
                    self.stdin_putc(mapped, &mut VIEWPORT.lock())
                }
            }

            _ => (),
        }
    }

    #[inline]
    /// respects `self.x_pos` and `self.y_pos` to start copying from
    pub fn draw_viewport(&mut self, viewport: &mut MutexGuard<Vec<u8>>) {
        let current_byte = (self.x_pos * self.y_pos) * self.info.bytes_per_pixel;
        let len = self.buffer.len();

        if current_byte > len {
            return self.scroll_down(true, viewport);
        }

        let start = self.viewport_start + current_byte;
        self.buffer[current_byte..].copy_from_slice(&viewport[start..len + start - current_byte]);
    }

    pub fn clear(&mut self, viewport: &mut MutexGuard<Vec<u8>>) {
        self.viewport_start = 0;

        self.x_pos = 0;
        self.y_pos = 0;
        viewport.truncate(self.buffer.len());
        viewport.fill(0);

        self.stdin_buffer = String::new();
        self.stdout_buffer = String::new();

        if self.mode == TerminalMode::Init {
            self.mode = TerminalMode::Stdin;
        }

        self.draw_viewport(viewport);
    }

    fn get_byte_offset(&self, x: usize, y: usize) -> usize {
        (y * self.info.stride + x) * self.info.bytes_per_pixel
    }

    #[inline]
    fn scroll_amount(&self) -> usize {
        self.info.stride * self.info.bytes_per_pixel * RASTER_HEIGHT.val()
    }

    #[inline]
    fn scroll_up(&mut self, viewport: &mut MutexGuard<Vec<u8>>) {
        let (old_y, old_x) = (self.y_pos, self.x_pos);
        self.y_pos = 0;
        self.x_pos = 0;

        let scroll_amount = self.scroll_amount();
        if self.viewport_start >= scroll_amount {
            self.viewport_start -= scroll_amount;

            self.draw_viewport(viewport)
        }

        (self.y_pos, self.x_pos) = (old_y, old_x);
    }

    // may the inline gods optimize this mess :pray: :pray: :pray:
    #[inline]
    /// if make_space it resizes viewport if possible if not removes the first line from buffer
    /// (shifts the buffer up by 1 line)
    fn scroll_down(&mut self, make_space: bool, viewport: &mut MutexGuard<Vec<u8>>) {
        let (mut old_y, old_x) = (self.y_pos, self.x_pos);

        let scroll_amount = self.scroll_amount();
        self.y_pos = 0;
        self.x_pos = 0;

        let len = viewport.len();

        // this should only execute if we were scrolling using page up and page down
        if !make_space && len >= self.viewport_start + scroll_amount + self.buffer.len() {
            self.viewport_start += scroll_amount;
            self.draw_viewport(viewport);
        } else if make_space {
            if len + scroll_amount <= self.buffer.len() * 4 {
                viewport.resize(len + scroll_amount, 0);

                self.viewport_start += scroll_amount;

                self.draw_viewport(viewport);
            } else {
                viewport.copy_within(scroll_amount..len, 0);
                viewport[len - scroll_amount..len].fill(0);

                old_y -= RASTER_HEIGHT.val();

                self.draw_viewport(viewport);
            }
        }

        (self.y_pos, self.x_pos) = (old_y, old_x);
    }

    fn newline(&mut self, viewport: &mut MutexGuard<Vec<u8>>) {
        self.y_pos += RASTER_HEIGHT.val();
        self.x_pos = 0;

        if self.y_pos * self.info.stride * self.info.bytes_per_pixel
            >= self.viewport_start + self.buffer.len()
        {
            self.scroll_down(true, viewport);
        }
    }

    pub fn remove_char(&mut self, c: char, viewport: &mut MutexGuard<Vec<u8>>) {
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
                self.set_pixel(self.x_pos + col, self.y_pos + row, 0, (0, 0, 0), viewport);
            }
        }
    }

    pub fn backspace(&mut self, viewport: &mut MutexGuard<Vec<u8>>) {
        self.stdin_buffer.pop(); // popping backspace
        let Some(char) = self.stdin_buffer.pop() else {
            return;
        };
        self.remove_char(char, viewport)
    }

    fn set_pixel(
        &self,
        x: usize,
        y: usize,
        intens: u32,
        color: (u8, u8, u8),
        viewport: &mut MutexGuard<Vec<u8>>,
    ) {
        let color = (color.0 as u32, color.1 as u32, color.2 as u32);

        let color = match self.info.pixel_format {
            PixelFormat::Rgb => [intens * color.0, intens * color.1, intens * color.2, 0],
            PixelFormat::Bgr => [intens * color.2, intens * color.1, intens * color.0, 0],
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

        viewport[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);

        unsafe {
            ptr::read_volatile(self.buffer.as_ptr()); // ensure buffer is not optimized away
        }
    }

    fn draw_char(
        &mut self,
        glyph: RasterizedChar,
        color: (u8, u8, u8),
        viewport: &mut MutexGuard<Vec<u8>>,
    ) {
        if (self.x_pos + glyph.width()) > self.info.stride {
            self.newline(viewport);
        }

        for (row, rows) in glyph.raster().iter().enumerate() {
            for (col, byte) in rows.iter().enumerate() {
                self.set_pixel(
                    self.x_pos + col,
                    self.y_pos + row,
                    *byte as u32,
                    color,
                    viewport,
                );
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

    pub fn putc(&mut self, c: char, color: (u8, u8, u8), viewport: &mut MutexGuard<Vec<u8>>) {
        match c {
            '\n' => {
                self.newline(viewport);
            }
            '\x08' => self.backspace(viewport),

            _ => {
                self.draw_char(Self::raster(c), color, viewport);
            }
        }
    }

    const INPUT_CHAR: (u8, u8, u8) = (170, 200, 30);
    pub fn stdin_putc(&mut self, c: char, viewport: &mut MutexGuard<Vec<u8>>) {
        // removing the _ if we are in stdin mode
        if self.mode == TerminalMode::Stdin && !self.stdin_buffer.is_empty() {
            self.remove_char('_', viewport)
        }

        self.stdin_buffer.push(c);

        self.putc(c, (255, 255, 255), viewport);

        // puts it back
        if self.mode == TerminalMode::Stdin && c != '\n' && !self.stdin_buffer.is_empty() {
            self.putc('_', Self::INPUT_CHAR, viewport);
        }

        self.draw_viewport(viewport)
    }

    fn write_slice(
        &mut self,
        str: &str,
        attributes: Attributes,
        viewport: &mut MutexGuard<Vec<u8>>,
    ) {
        let old_mode = self.mode;
        self.mode = TerminalMode::Stdout;
        self.stdout_buffer.push_str(str);

        for c in str.chars() {
            self.putc(c, attributes.fg, viewport);
        }

        self.draw_viewport(viewport);
        self.mode = old_mode
    }

    pub fn write_es(
        &mut self,
        escape_seq: NaviTTES,
        default_attributes: Attributes,
        viewport: &mut MutexGuard<Vec<u8>>,
    ) {
        match escape_seq {
            NaviTTES::Slice(s) => self.write_slice(s, default_attributes, viewport),
            NaviTTES::OwnedSlice(s) => self.write_slice(&s, default_attributes, viewport),

            NaviTTES::NaviESS(escape_seqs) => {
                for escape_seq in escape_seqs {
                    self.write_es(escape_seq, default_attributes.clone(), viewport)
                }
            }
            NaviTTES::NaviES(attributes, seq) => {
                let attributes = Attributes::from_list(&attributes, default_attributes);

                self.write_es(*seq, attributes, viewport)
            }
        }
    }
    pub fn write(&mut self, str: &str) {
        let viewport = &mut VIEWPORT.lock();

        let parsed = NaviTTES::parse_str(str);
        let mut default_attributes = Attributes::default();
        default_attributes.fg = WRITE_COLOR;

        self.write_es(parsed, default_attributes, viewport)
    }
}
