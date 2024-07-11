#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(asm_const)]

macro_rules! s {
    ($str: expr) => {
        $str.as_ptr()
    };
}
mod arch;
mod kernel;

use core::{arch::asm, panic::PanicInfo};

use kernel::vga::{kerr, kput, kwrite};

use crate::kernel::vga;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kput(b'\n'); // ensures correctly formated panic

    kerr(s!(b"Kernel panic: \0"));
    kerr(info.message().as_str().unwrap().as_ptr());
    kerr(s!(b"\ncannot continue execution kernel will now hang\0\n"));
    loop {}
}

pub fn strlen(cstr: *const u8) -> usize {
    let mut len = 0;

    while unsafe { *cstr.offset(len as isize) } != b'\0' {
        len += 1;
    }
    len
}

pub extern "C" fn kinit() {
    vga::init_vga();

    arch_init!(); // macro is defined for each arch
}

#[no_mangle]
fn kmain(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    kinit();

    // kwrite(s!(b"Hello, world!\n\0"));
    loop {}
}
bootloader_api::entry_point!(kmain);
