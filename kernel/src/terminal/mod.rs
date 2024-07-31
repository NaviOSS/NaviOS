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
