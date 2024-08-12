pub mod framebuffer;
pub mod navitts;

use core::fmt;

use alloc::{string::String, vec::Vec};
use framebuffer::TerminalMode;

use crate::{globals::terminal, print, println, serial};

#[doc(hidden)]
#[no_mangle]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    terminal().write_fmt(args).unwrap();
}

pub fn readln() -> String {
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

    print!("you sure? y\\N: ");
    let confirm = readln();

    if confirm.to_uppercase() == "Y" {
        terminal().clear()
    }
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

// badly written shell process
pub fn shell() {
    serial!("shell!\n");
    // waits until we leave init mode which happens on the first terminal().clear()
    while terminal().mode == TerminalMode::Init {}
    print!(
        r"\[fg: (0, 255, 0) ||
 _   _             _  ____   _____
| \ | |           (_)/ __ \ / ____|
|  \| | __ ___   ___| |  | | (___
| . ` |/ _` \ \ / / | |  | |\___ \
| |\  | (_| |\ V /| | |__| |____) |
|_| \_|\__,_| \_/ |_|\____/|_____/
||]"
    );
    print!("\\[fg: (255, 255, 255) ||\nwelcome to NaviOS!\ntype help or ? for a list of avalible commands\n||]");

    loop {
        terminal().enter_stdin()
    }
}
