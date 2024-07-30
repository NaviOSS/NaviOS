use spin::MutexGuard;

use core::{fmt, ptr};

use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use noto_sans_mono_bitmap::{FontWeight, RasterHeight, RasterizedChar};

use crate::{
    drivers::keyboard::{Key, KeyCode},
    utils::Locked,
};

use super::navitts::NaviTTES;

const RASTER_HEIGHT: RasterHeight = RasterHeight::Size20;
const WRITE_COLOR: Color = (222, 255, 30);

pub type Color = (u32, u32, u32);

pub struct Terminal<'a> {
    row: usize,
    column: usize,
    buffer: &'a mut [u8],
    viewport_start: usize,
    pub info: FrameBufferInfo,
    pub x_pos: usize,
    pub y_pos: usize,
}
const MAX_VIEWPORT_LEN: usize = 4000000 * 8;

static VIEWPORT: Locked<[u8; MAX_VIEWPORT_LEN]> = Locked::new([0u8; MAX_VIEWPORT_LEN]); // TODO use
                                                                                        // a vec instead after we finish scrolling
fn viewport() -> MutexGuard<'static, [u8; MAX_VIEWPORT_LEN]> {
    VIEWPORT.inner.lock()
}

impl<'a> Terminal<'a> {
    pub fn init(frame_buffer: &'static mut FrameBuffer) -> Self {
        let frame_buffer = frame_buffer;
        let info = frame_buffer.info();
        let buffer = frame_buffer.buffer_mut();

        for i in 0..buffer.len() {
            buffer[i] = 0;
        }

        Self {
            row: 0,
            column: 0,
            buffer,
            viewport_start: 0,
            info,
            x_pos: 0,
            y_pos: 0,
        }
    }

    pub fn on_key_pressed(&mut self, key: Key) {
        match key.code {
            KeyCode::PageDown => self.scroll_down(),
            KeyCode::PageUp => self.scroll_up(),
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
            &viewport()[self.viewport_start..self.buffer.len() + self.viewport_start],
        );
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

    fn scroll_down(&mut self) {
        let scroll_amount = self.scroll_amount();
        if viewport().len() - 1 >= (self.viewport_start + scroll_amount + self.buffer.len()) {
            self.viewport_start += scroll_amount;
            /*             self.screen.resize(self.screen.len() + scroll_amount, 0); */
            self.draw_viewport()
        }
    }

    fn newline(&mut self) {
        self.y_pos += RASTER_HEIGHT.val();
        self.x_pos = 0;
    }

    fn set_pixel(&mut self, x: usize, y: usize, intens: u32, color: (u32, u32, u32)) {
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

        viewport()[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);

        unsafe {
            ptr::read_volatile(self.buffer.as_ptr()); // ensure buffer is not optimized away
        }
    }

    fn draw_char(&mut self, glyph: RasterizedChar, color: (u32, u32, u32)) {
        if (self.x_pos + glyph.width()) > self.width() {
            self.newline();
        }

        if self.y_pos * self.info.stride * self.info.bytes_per_pixel
            >= self.viewport_start + self.buffer.len()
        {
            self.scroll_down();
        }

        for (row, rows) in glyph.raster().iter().enumerate() {
            for (col, byte) in rows.iter().enumerate() {
                self.set_pixel(self.x_pos + col, self.y_pos + row, *byte as u32, color);
            }
        }

        self.x_pos += glyph.width();
    }

    pub fn putc(&mut self, c: char, color: (u32, u32, u32)) {
        match c {
            '\n' => {
                self.newline();
            }

            _ => {
                let null =
                    noto_sans_mono_bitmap::get_raster('N', FontWeight::Regular, RASTER_HEIGHT)
                        .unwrap();
                let glyph = noto_sans_mono_bitmap::get_raster(c, FontWeight::Bold, RASTER_HEIGHT)
                    .unwrap_or(null);

                self.draw_char(glyph, color);
            }
        }
    }

    fn write_slice(&mut self, str: &str, color: (u32, u32, u32)) {
        for c in str.chars() {
            self.putc(c, color);
        }

        self.draw_viewport();
    }
    pub fn write(&mut self, str: &str, color: (u32, u32, u32)) {
        let parsed = NaviTTES::parse_str(str);
        match parsed {
            NaviTTES::Slice(s) => self.write_slice(s, color),
            u => todo!("{:#?}", u),
        }
    }
}

impl fmt::Write for Terminal<'static> {
    // i can add color escapes later on like parsing \(u8, u8, u8)str$ as coloring str into (u8, u8, u8)
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s, WRITE_COLOR);
        Ok(())
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        self.putc(c, WRITE_COLOR);
        Ok(())
    }
}
