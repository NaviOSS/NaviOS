pub mod framebuffer;
pub mod navitts;

use core::{fmt, str};

use alloc::{string::String, vec::Vec};
use framebuffer::TerminalMode;

use crate::{arch, globals::terminal, print, println, scheduler, serial};

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

fn help(args: Vec<&str>) {
    if args.len() != 1 {
        println!("{}: expected 1 arg", args[0]);
        return;
    }

    println!(
        "info:
    scroll up using `page up` and scroll down using `page down`,
    this shell supports string slices starting with '\"'
commands:
    help, ?: displays this
    echo `text`: echoes back text
    clear: clears the screen

    shutdown: shutdowns qemu and bochs only for now
    reboot: force-reboots the PC for now

    plist: list the avalible process' pids and names
    pkill `pid`: kills a process with pid `pid`
    pkillall `name`: kills all processs with name `name`"
    );
}

fn clear(args: Vec<&str>) {
    if args.len() != 1 {
        println!("{}: expected 0 args", args[0]);
        return;
    }

    print!("you sure? y\\N: ");
    let confirm = readln();

    if confirm.to_uppercase() == "Y" {
        terminal().clear()
    }
}

fn reboot_cmd(args: Vec<&str>) {
    if args.len() != 1 {
        println!("{}: expected 0 args", args[0]);
        return;
    }

    arch::power::reboot();
}

fn shutdown_cmd(args: Vec<&str>) {
    if args.len() != 1 {
        println!("{}: expected 0 args", args[0]);
        return;
    }

    arch::power::shutdown();
}

fn plist(args: Vec<&str>) {
    if args.len() != 1 {
        println!("{}: expected 0 args", args[0]);
        return;
    }

    let mut process_list: Vec<(u64, [u8; 64])> = Vec::new();

    let mut current = &scheduler().head;
    process_list.push((current.pid, current.name));

    while let Some(ref process) = current.next {
        process_list.push((process.pid, process.name));
        current = process;
    }

    println!("{} process(s) is currently running:", process_list.len());
    println!("name:  pid");
    for (pid, name) in process_list {
        let mut name = name.to_vec();
        while name.last() == Some(&0) {
            name.pop();
        }

        println!("{}:  {}", str::from_utf8(&name).unwrap(), pid);
    }
}

fn pkill(args: Vec<&str>) {
    if args.len() != 2 {
        println!("{}: expected 1 arg which is the pid", args[0]);
        return;
    }

    let pid = args[1].parse();
    if pid.is_err() {
        println!("couldn't parse pid make sure it is a vaild number...");
        return;
    }

    let pid = pid.unwrap();

    if pid == 0 {
        println!("it looks like you are trying to kill us sadly this doesn't work duo to a bug which will never be fixed\nwe will try to do that anyways you monster!")
    }

    scheduler()
        .pkill(pid)
        .unwrap_or_else(|_| println!("couldn't find a process with pid `{}`", pid));
}

fn pkillall(args: Vec<&str>) {
    if args.len() != 2 {
        println!("{}: expected one arg which is the process name", args[0]);
        return;
    }

    let name = args[1].as_bytes();

    scheduler()
        .pkillall(name)
        .unwrap_or_else(|_| println!("couldn't find a process with name `{}`", args[1]));
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
        "reboot" => return reboot_cmd(command),
        "shutdown" => return shutdown_cmd(command),

        "plist" => plist(command),
        "pkill" => pkill(command),
        "pkillall" => pkillall(command),
        _ => println!("unknown command {}", command[0]),
    }

    terminal().enter_stdin()
}

// badly written shell process
pub fn shell() {
    serial!("shell!\n");
    // waits until we leave init mode which happens on the first terminal().clear()
    while terminal().mode != TerminalMode::Stdin {}
    serial!("entering stdin... {:?}\n", terminal().mode);

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
