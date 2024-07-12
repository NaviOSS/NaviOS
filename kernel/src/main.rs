#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(asm_const)]

mod arch;
mod terminal;

use core::panic::PanicInfo;

#[allow(unused_imports)]
use terminal::framebuffer::{kerr, kput, kwrite, kwriteln, Terminal};
#[allow(dead_code)]
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kerr("Kernel panic: ");
    kerr(info.message().as_str().unwrap());
    kerr("\ncannot continue execution kernel will now hang");
    loop {}
}

pub fn strlen(cstr: *const u8) -> usize {
    let mut len = 0;

    while unsafe { *cstr.offset(len as isize) } != b'\0' {
        len += 1;
    }
    len
}

pub extern "C" fn kinit(boot_info: &'static mut bootloader_api::BootInfo) {
    // initing terminal
    let terminal = Terminal::init(boot_info.framebuffer.as_mut().unwrap());
    unsafe {
        TERMINAL = Some(terminal);
    }
    // initing the arch
    arch_init!(); // macro is defined for each arch
}

static mut TERMINAL: Option<Terminal> = None;
#[no_mangle]
fn kmain(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    kinit(boot_info);

    kwriteln("Hello, world!");
    // panic!("kernel works finally now you can hang in peace");
    loop {}
}

bootloader_api::entry_point!(kmain);
