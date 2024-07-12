use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use noto_sans_mono_bitmap::{FontWeight, RasterHeight, RasterizedChar};

use crate::TERMINAL;

const CHAR_WIDTH: usize = 8;
const CHAR_HEIGHT: usize = 16;

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

// safe wrappers around TERMINAL
#[no_mangle]
pub fn kwrite(str: &str) {
    unsafe { TERMINAL.as_mut().unwrap().write(str, (100, 22, 200)) }
}

#[no_mangle]
pub fn kput(c: char) {
    unsafe { TERMINAL.as_mut().unwrap().putc(c, (100, 22, 200)) }
}

#[no_mangle]
pub fn kerr(str: &str) {
    unsafe { TERMINAL.as_mut().unwrap().write(str, (200, 0, 0)) }
}

#[no_mangle]
pub fn kwriteln(str: &str) {
    kwrite(str);
    kput('\n');
}

// gpt4 generated
pub fn u64_to_hex_array(value: u64) -> [u8; 18] {
    let mut hex_array = [b' '; 18];
    hex_array[0] = b'0';
    hex_array[1] = b'x';
    let i = hex_array.len() - 2;

    for i in 0..i {
        let shr = (15 - i) * 4;
        let nibble = ((value >> shr) & 0xF) as u8;

        hex_array[i + 2] = match nibble {
            0..=9 => b'0' + nibble as u8,
            _ => b'A' + (nibble - 10) as u8,
        };
    }

    hex_array
}

pub fn kwrite_hex(hex: u64) {
    let arr = u64_to_hex_array(hex);
    kwriteln(core::str::from_utf8(&arr).unwrap())
}
