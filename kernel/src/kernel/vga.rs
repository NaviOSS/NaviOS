use core::ptr;

use crate::strlen;

const VGA_BUFFER: *mut u16 = 0xb8000 as *mut u16;

static mut TERMINAL_ROW: usize = 0;
static mut TERMINAL_COLUMN: usize = 0;
const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

#[allow(dead_code)]
#[repr(u8)]
pub enum VGAColor {
    Black = 0,
    Blue = 1,
    Cyan = 3,
    Red = 4,
    White = 15,
}

pub const fn vga_entry_color(background: VGAColor, foreground: VGAColor) -> u8 {
    foreground as u8 | (background as u8) << 4
}

pub const fn vga_entry(char: u8, color: u8) -> u16 {
    char as u16 | ((color as u16) << 8)
}

pub fn terminal_put(entry: u16) {
    if entry as u8 == b'\n' {
        unsafe {
            TERMINAL_ROW += 1;
            TERMINAL_COLUMN = 0;
        }
        return;
    }

    unsafe {
        *VGA_BUFFER.offset((TERMINAL_COLUMN + VGA_WIDTH * TERMINAL_ROW) as isize) = entry;
        TERMINAL_COLUMN += 1;
    }
}

pub fn terminal_put_str(str: *const u8, color: u8) {
    let len = strlen(str);
    for i in 0..len {
        let char = unsafe { *str.offset(i as isize) };

        terminal_put(vga_entry(char, color));
    }
}

pub extern "C" fn kwrite(str: *const u8) {
    terminal_put_str(str, vga_entry_color(VGAColor::Black, VGAColor::Cyan));
}

pub extern "C" fn kerr(str: *const u8) {
    terminal_put_str(str, vga_entry_color(VGAColor::Black, VGAColor::Red))
}

pub extern "C" fn kput(char: u8) {
    terminal_put(vga_entry(
        char,
        vga_entry_color(VGAColor::Black, VGAColor::White),
    ));
}

pub extern "C" fn init_vga() {
    let blank = vga_entry(b' ', vga_entry_color(VGAColor::White, VGAColor::White));

    for i in 0..(VGA_WIDTH * VGA_HEIGHT) {}
    unsafe {
        *VGA_BUFFER.offset(0) = 0;
    }
}

// gpt4 generated
pub fn u32_to_hex_array(value: u32) -> [u8; 11] {
    let mut hex_array = [0u8; 11];
    hex_array[0] = b'0';
    hex_array[1] = b'x';

    for i in 0..8 {
        let nibble = (value >> (28 - i * 4)) & 0xF;
        hex_array[i + 2] = match nibble {
            0..=9 => b'0' + nibble as u8,
            _ => b'a' + (nibble - 10) as u8,
        };
    }

    hex_array[10] = b'\0';

    hex_array
}

pub fn write_hex(hex: u32) {
    kwrite(u32_to_hex_array(hex).as_ptr());
    kput(b'\n');
}
