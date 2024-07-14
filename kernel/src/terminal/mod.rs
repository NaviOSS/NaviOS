pub mod framebuffer;
pub mod navitts;

use crate::TERMINAL;
use core::fmt;

#[doc(hidden)]
#[no_mangle]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    unsafe {
        TERMINAL.as_mut().unwrap().write_fmt(args).unwrap();
    }
}

// safe wrappers around TERMINAL

#[no_mangle]
pub fn kerr(str: &str) {
    unsafe { TERMINAL.as_mut().unwrap().write(str, (200, 0, 0)) }
}
