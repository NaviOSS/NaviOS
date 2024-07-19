pub mod framebuffer;
pub mod navitts;

use core::fmt;

use crate::globals::terminal;

#[doc(hidden)]
#[no_mangle]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    terminal().write_fmt(args).unwrap();
}

// safe wrappers around TERMINAL

#[no_mangle]
pub fn kerr(str: &str) {
    terminal().write(str, (200, 0, 0))
}
