/// TODO: fix the framebuffer acting like BGR even tho it is RGB
use alloc::{string::String, vec::Vec};
use lazy_static::lazy_static;
use spin::Mutex;

use core::{
    fmt::Write,
    mem::MaybeUninit,
    str::{self, Chars},
};

use noto_sans_mono_bitmap::{FontWeight, RasterHeight, RasterizedChar};

use crate::{
    debug,
    drivers::keyboard::{Key, KeyCode, KeyFlags},
    kernel,
    memory::align_down,
    println, serial, terminal,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalMode {
    Init,
    Stdout,
    Stdin,
    Panic,
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

type RGB = (u8, u8, u8);

const RASTER_HEIGHT: RasterHeight = RasterHeight::Size20;
const WRITE_COLOR: RGB = (255, 255, 255);
const BG_COLOR: RGB = (0, 0, 0);

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
    pub draw_from_x: usize,
    pub draw_from_y: usize,
    /// wether or not the current panic happend in another panic because of the terminal
    pub panicked: bool,

    /// changed by ansii escape sequnceses
    /// currently only `\e[38;2;<r>;<g>;<b>m` is supported
    pub text_fg: (u8, u8, u8),
    /// currently only `\e[48;2;<r>;<g>;<b>m` is supported
    pub text_bg: (u8, u8, u8),
    pub ready: bool,
}

lazy_static! {
    pub static ref VIEWPORT: Mutex<Vec<u8>> = Mutex::new(Vec::new());
}

impl Terminal {
    /// must finish the init using init_finish after memory is ready
    pub fn init() {
        debug!(Terminal, "initing (framebuffer) ...");
        let (buffer, info) = crate::limine::get_framebuffer();

        for i in 0..buffer.len() {
            buffer[i] = 0;
        }

        serial!("{} {:#?}\n", buffer.len(), info);

        let this = Self {
            buffer,
            viewport_start: 0,

            mode: TerminalMode::Init,
            stdin_buffer: String::new(),
            stdout_buffer: String::new(),

            info,
            x_pos: 0,
            y_pos: 0,
            draw_from_x: 0,
            draw_from_y: 0,
            panicked: false,
            text_fg: WRITE_COLOR,
            text_bg: BG_COLOR,
            ready: false,
        };

        kernel().terminal = MaybeUninit::new(this);
        debug!(Terminal, "done ...");
    }

    pub fn init_finish() {
        terminal().ready = true;
        VIEWPORT.lock().resize(terminal().buffer.len(), 0);
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
    pub fn draw_viewport(&mut self, viewport: &mut [u8]) {
        let mut current_byte = self.get_byte_offset(self.draw_from_x, self.draw_from_y);

        let len = self.buffer.len();

        if current_byte > len {
            current_byte = 0;
        }

        let start = self.viewport_start + current_byte;
        self.buffer[current_byte..].copy_from_slice(&viewport[start..len + start - current_byte]);

        self.draw_from_x = self.x_pos;
        self.draw_from_y = self.y_pos;
    }

    pub fn clear(&mut self, viewport: &mut [u8]) {
        self.viewport_start = 0;

        self.x_pos = 0;
        self.y_pos = 0;
        viewport.fill(0);

        self.stdin_buffer = String::new();
        self.stdout_buffer = String::new();

        if self.mode == TerminalMode::Init {
            self.mode = TerminalMode::Stdin;
        }

        self.draw_viewport(viewport);
    }

    #[inline(always)]
    fn get_byte_offset(&self, x: usize, y: usize) -> usize {
        (y * self.info.stride + x) * self.info.bytes_per_pixel
    }

    #[inline(always)]
    fn scroll_amount(&self) -> usize {
        self.info.stride * self.info.bytes_per_pixel * RASTER_HEIGHT.val()
    }

    #[inline]
    fn scroll_up(&mut self, viewport: &mut Vec<u8>) {
        let scroll_amount = self.scroll_amount();
        if self.viewport_start >= scroll_amount {
            self.viewport_start -= scroll_amount;
            self.draw_viewport(viewport)
        }
    }

    #[inline(always)]
    fn force_scroll_down(&mut self, viewport: &mut [u8]) {
        let scroll_amount = self.scroll_amount();
        let len = viewport.len();

        viewport.copy_within(scroll_amount..len, 0);
        viewport[len - scroll_amount..len].fill(0);
        self.x_pos = 0;
        self.y_pos -= RASTER_HEIGHT.val();
    }

    // may the inline gods optimize this mess :pray: :pray: :pray:
    #[inline]
    /// if make_space it resizes viewport if possible if not removes the first line from buffer
    /// (shifts the buffer up by 1 line)
    fn scroll_down(&mut self, make_space: bool, viewport: &mut Vec<u8>) {
        let scroll_amount = self.scroll_amount();
        let len = viewport.len();

        // this should only execute if we were scrolling using page up and page down
        if !make_space && len >= self.viewport_start + scroll_amount + self.buffer.len() {
            self.viewport_start += scroll_amount;
        } else if make_space {
            if len + scroll_amount <= self.buffer.len() * 4 {
                viewport.resize(len + scroll_amount, 0);
                self.viewport_start += scroll_amount;
            } else {
                self.force_scroll_down(viewport);
            }
        }
        self.draw_viewport(viewport);
    }

    fn newline(&mut self, viewport: &mut Vec<u8>) {
        self.y_pos += RASTER_HEIGHT.val();
        self.x_pos = 0;

        if self.y_pos * self.info.stride * self.info.bytes_per_pixel
            >= self.viewport_start + self.buffer.len()
        {
            self.scroll_down(true, viewport);
        }
    }

    pub fn remove_char(&mut self, c: char, viewport: &mut [u8]) {
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

    pub fn backspace(&mut self, viewport: &mut [u8]) {
        self.stdin_buffer.pop(); // popping backspace
        let Some(char) = self.stdin_buffer.pop() else {
            return;
        };
        self.remove_char(char, viewport)
    }

    #[inline]
    fn set_pixel(&self, x: usize, y: usize, intens: u32, color: (u8, u8, u8), viewport: &mut [u8]) {
        let color = (color.0 as u32, color.1 as u32, color.2 as u32);

        let color = match self.info.pixel_format {
            PixelFormat::Bgr => [intens * color.0, intens * color.1, intens * color.2, 0],
            PixelFormat::Rgb => [intens * color.2, intens * color.1, intens * color.0, 0],
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
    }

    #[inline]
    /// UNCHECKED doesn't do boundaries checks!
    /// will panic if the glyph is outside boundaries
    fn draw_char(&mut self, glyph: RasterizedChar, color: (u8, u8, u8), viewport: &mut [u8]) {
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

    #[inline(always)]
    fn raster(c: char) -> RasterizedChar {
        let null =
            noto_sans_mono_bitmap::get_raster('N', FontWeight::Regular, RASTER_HEIGHT).unwrap();
        let glyph =
            noto_sans_mono_bitmap::get_raster(c, FontWeight::Bold, RASTER_HEIGHT).unwrap_or(null);
        glyph
    }

    /// puts a character without extending viewport
    pub fn dputc(&mut self, c: char, color: (u8, u8, u8), viewport: &mut [u8]) {
        let glyph = Self::raster(c);
        if (self.y_pos + glyph.height()) * self.info.stride * self.info.bytes_per_pixel
            >= self.viewport_start + self.buffer.len()
        {
            self.force_scroll_down(viewport);
        }

        match c {
            '\n' => {
                self.y_pos += RASTER_HEIGHT.val();
                self.x_pos = 0;

                if self.y_pos * self.info.stride * self.info.bytes_per_pixel
                    >= self.viewport_start + self.buffer.len()
                {
                    self.force_scroll_down(viewport);
                }
            }
            _ => self.draw_char(glyph, color, viewport),
        }
    }

    #[inline(always)]
    fn putc(&mut self, c: char, color: RGB, viewport: &mut Vec<u8>) {
        let glyph = Self::raster(c);

        if (self.x_pos + glyph.width()) > self.info.stride {
            self.newline(viewport);
        }

        if (self.y_pos + glyph.height()) * self.info.stride * self.info.bytes_per_pixel
            >= self.viewport_start + self.buffer.len()
        {
            self.scroll_down(true, viewport);
        }

        match c {
            '\n' => self.newline(viewport),
            '\x08' => self.backspace(viewport),
            _ => self.draw_char(glyph, color, viewport),
        }
    }

    const INPUT_CHAR: (u8, u8, u8) = (170, 200, 30);
    pub fn stdin_putc(&mut self, c: char, viewport: &mut Vec<u8>) {
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

    fn parse_ansii_rgb(bytes: &[u8]) -> Result<RGB, ()> {
        let mut rgb = 0u32;

        let mut current_color = [48u8; 3];
        let mut current_color_index = 0;

        let mut current_rgb_index = 0;

        for byte in bytes {
            if *byte == b'm' {
                break;
            }

            if current_rgb_index >= 3 {
                return Err(());
            }

            if *byte == b';' {
                let str = unsafe { str::from_utf8_unchecked(&current_color) };
                let int = u8::from_str_radix(str, 10).unwrap();

                rgb |= (int as u32) << ((3 - current_rgb_index) * 8);

                current_rgb_index += 1;

                current_color = *b"000";
                current_color_index = 0;
                continue;
            }

            if current_color_index >= current_color.len() {
                return Err(());
            }

            current_color[current_color_index] = *byte;
            current_color_index += 1;
        }

        let rgb = ((rgb >> 24) as u8, (rgb >> 16) as u8, (rgb >> 8) as u8);

        Ok(rgb)
    }

    fn parse_ansi(&mut self, bytes: [u8; 14]) -> Result<(), ()> {
        match &bytes[0..2] {
            b"0m" => {
                self.text_fg = WRITE_COLOR;
                self.text_bg = BG_COLOR;
            }

            b"38" => {
                if bytes[2] != b';' {
                    return Err(());
                }

                match bytes[3] {
                    b'2' => {
                        if bytes[4] != b';' {
                            return Err(());
                        }

                        let rgb = Self::parse_ansii_rgb(&bytes[5..])?;
                        self.text_fg = rgb;
                    }
                    _ => (),
                }
            }

            b"48" => {
                if bytes[2] != b';' {
                    return Err(());
                }

                match bytes[3] {
                    b'2' => {
                        if bytes[4] != b';' {
                            return Err(());
                        }

                        let rgb = Self::parse_ansii_rgb(&bytes[5..])?;
                        self.text_bg = rgb;
                    }
                    _ => (),
                }
            }
            _ => (),
        }

        Ok(())
    }

    fn handle_ansii_escape(&mut self, chars: &mut Chars) -> Result<(), ()> {
        if chars.next() != Some('[') {
            return Err(());
        }

        let mut bytes = [0u8; 14];
        let mut len = 0;
        while let Some(c) = chars.next() {
            len += 1;
            bytes[len - 1] = c as u8;

            if len == bytes.len() || c == 'm' {
                break;
            }
        }

        if bytes[len - 1] != b'm' {
            return Err(());
        }

        self.parse_ansi(bytes)
    }

    fn write_slice(&mut self, str: &str, viewport: &mut Vec<u8>) {
        let old_mode = self.mode;
        self.mode = TerminalMode::Stdout;
        self.stdout_buffer.push_str(str);

        let mut chars = str.chars();

        while let Some(c) = chars.next() {
            match c {
                '\x1B' => _ = self.handle_ansii_escape(&mut chars),
                _ => self.putc(c, self.text_fg, viewport),
            }
        }

        self.draw_viewport(viewport);
        self.mode = old_mode
    }

    /// directly write to buffer
    pub fn dwrite(&mut self, str: &str) {
        let (buffer, _) = crate::limine::get_framebuffer();
        let mut chars = str.chars();

        while let Some(c) = chars.next() {
            match c {
                '\x1B' => _ = self.handle_ansii_escape(&mut chars),
                _ => self.dputc(c, self.text_fg, buffer),
            }
        }
    }

    pub fn write(&mut self, str: &str) {
        match self.mode {
            TerminalMode::Panic => self.dwrite(str),
            _ => {
                let viewport = &mut VIEWPORT.lock();

                self.write_slice(str, viewport)
            }
        }
    }

    #[allow(unused)]
    pub fn enter_panic(&mut self) {
        self.x_pos = 0;
        self.y_pos = 0;
        self.buffer.fill(0);

        self.mode = TerminalMode::Panic;
        self.ready = true;
        println!("\x1B[38;2;255;0;0mPANIC MODE");
    }
}

impl Write for Terminal {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s);
        Ok(())
    }
}
