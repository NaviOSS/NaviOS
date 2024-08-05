pub mod framebuffer;
pub mod navitts;

use core::fmt;

use alloc::{string::String, vec::Vec};

use crate::{globals::terminal, println};

#[doc(hidden)]
#[no_mangle]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    terminal().write_fmt(args).unwrap();
}

pub fn process_command(command: &mut String) {
    let cmd = command.clone();
    let cmd: Vec<&str> = cmd.split(' ').collect();
    command.clear();

    match cmd[0] {
        _ => println!("unknown command {}", cmd[0]),
    }
}
