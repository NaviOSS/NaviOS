pub mod framebuffer;
pub mod navitts;

use core::fmt;

use alloc::{string::String, vec::Vec};
use framebuffer::TerminalMode;

use crate::{globals::terminal, println};

#[doc(hidden)]
#[no_mangle]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    terminal().write_fmt(args).unwrap();
}

/// doesnt work rn
pub fn _readln() -> String {
    let old_mode = terminal().mode;
    terminal().mode = TerminalMode::Stdin;

    loop {
        if terminal().stdin_buffer.ends_with('\n') {
            terminal().stdin_buffer.pop();

            let buffer = terminal().stdin_buffer.clone();
            terminal().stdin_buffer.clear();

            terminal().mode = old_mode;
            return buffer;
        }
    }
}

pub fn echo(args: Vec<&str>) {
    if args.len() != 2 {
        println!("echo: expected 2 args");
        return;
    }

    println!("{}", args[1]);
}

pub fn help(args: Vec<&str>) {
    if args.len() != 1 {
        println!("{}: expected 1 arg", args[0]);
        return;
    }

    println!(
        "commands:
    help, ?: displays this
    echo: echoes back text
    clear: clears the screen"
    );
}

fn clear(args: Vec<&str>) {
    if args.len() != 1 {
        println!("{}: expected 1 arg", args[0]);
        return;
    }

    terminal().clear()
}

// bad shell
pub fn process_command(command: String) {
    let mut unterminated_str_slice = false;
    let command: Vec<&str> = command
        .split(|c| {
            if unterminated_str_slice && c == '"' {
                unterminated_str_slice = false;
            } else if c == '"' {
                unterminated_str_slice = true;
            }

            (c == ' ') && (!unterminated_str_slice)
        })
        .collect();

    if unterminated_str_slice {
        println!("unterminated string \" expected");
        return terminal().enter_stdin();
    }

    match command[0] {
        "echo" => echo(command),
        "?" | "help" => help(command),
        "clear" => return clear(command),
        _ => println!("unknown command {}", command[0]),
    }

    terminal().enter_stdin()
}
