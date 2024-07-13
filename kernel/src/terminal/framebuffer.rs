use core::{fmt, ptr};

use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use noto_sans_mono_bitmap::{FontWeight, RasterHeight, RasterizedChar};

const CHAR_WIDTH: usize = 8;
const CHAR_HEIGHT: usize = 16;
pub type Color = (u32, u32, u32);

pub struct Terminal<'a> {
    row: usize,
    column: usize,
    buffer: &'a mut [u8],
    info: FrameBufferInfo,
    x_pos: usize,
    y_pos: usize,
}

impl<'a> Terminal<'a> {
    pub fn init(frame_buffer: &'a mut FrameBuffer) -> Self {
        let info = frame_buffer.info();
        let buffer = frame_buffer.buffer_mut();

        for i in 0..buffer.len() {
            buffer[i] = 0;
        }

        Self {
            row: 0,
            column: 0,
            buffer,
            info,
            x_pos: 0,
            y_pos: 0,
        }
    }

    fn width(&self) -> usize {
        self.info.width
    }

    fn height(&self) -> usize {
        self.info.height
    }

    fn newline(&mut self) {
        self.y_pos += RasterHeight::Size24.val();
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

        let pixel_offset = y * self.info.stride + x;
        let bytes_per_pixel = self.info.bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;

        // i dont know why this works just did some random stuff but it does!
        let color = [
            (color[0] >> 8 & 0xff) as u8,
            (color[1] >> 8 & 0xff) as u8,
            (color[2] >> 8 & 0xff) as u8,
            (color[3] >> 8 & 0xff) as u8,
        ];

        self.buffer[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
        // ensure buffer is not optimized away
        unsafe {
            ptr::read_volatile(self.buffer.as_ptr());
        }
    }

    fn draw_char(&mut self, glyph: RasterizedChar, color: (u32, u32, u32)) {
        if (self.x_pos + glyph.width()) > self.width() {
            self.newline();
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
                let null = noto_sans_mono_bitmap::get_raster(
                    'N',
                    FontWeight::Regular,
                    RasterHeight::Size24,
                )
                .unwrap();
                let glyph =
                    noto_sans_mono_bitmap::get_raster(c, FontWeight::Bold, RasterHeight::Size24)
                        .unwrap_or(null);

                self.draw_char(glyph, color);
            }
        }
    }

    pub fn write(&mut self, str: &str, color: (u32, u32, u32)) {
        for c in str.chars() {
            self.putc(c, color);
        }
    }
}

const WRITE_COLOR: Color = (100, 22, 200);
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
