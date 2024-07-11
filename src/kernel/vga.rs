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
    for i in 0..VGA_WIDTH * VGA_HEIGHT {
        unsafe {
            *VGA_BUFFER.offset(i as isize) = 0;
        }
    }
}
