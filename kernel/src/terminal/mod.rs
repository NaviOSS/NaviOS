pub mod framebuffer;
pub mod navitts;

use core::{fmt, str};

use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
    vec::Vec,
};
use framebuffer::TerminalMode;

use crate::{
    arch,
    drivers::vfs::{vfs, FS},
    globals::terminal,
    print, println, scheduler, serial,
};

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
        println!("echo: expected 1 arg");
        return;
    }

    println!("{}", args[1]);
}

fn help(args: Vec<&str>) {
    if args.len() != 1 {
        println!("{}: expected 0 args", args[0]);
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
    pkillall `name`: kills all processs with name `name`

    touch `new_file_path`: creates a new empty file, the path of the new file would be equal to `new_file_path`
    mkdir `new_dir_path`: creates a new empty directory, the path of the new directory would be equal to `new_dir_path` 
    ls: lists all files and directories in the current dir
    cd `target_dir`: changes the current dir to `target_dir`

    cat `src_files`: echoes the contents of a file
    write `target_file` `src_text`: writes `src_text` to `target_file`"
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
        println!("{}: expected the pid", args[0]);
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
        println!("{}: expected the process name", args[0]);
        return;
    }

    let name = args[1].as_bytes();

    scheduler()
        .pkillall(name)
        .unwrap_or_else(|_| println!("couldn't find a process with name `{}`", args[1]));
}

/// returns the absloutel path of a given path respecting `Ternminal.current_dir`
/// returned path won't end with / if it is a directory
fn get_path(path: &str) -> String {
    for c in path.chars() {
        if c == '/' || c == '\\' {
            break;
        }

        if c == ':' {
            return path.to_string();
        }
    }

    return terminal().current_dir.clone() + path;
}

fn mkdir(args: Vec<&str>) {
    if args.len() != 2 {
        println!("{}: expected just the new dir path", args[0]);
        return;
    }

    let path = get_path(args[1]);

    let mut spilt: Vec<&str> = path.split(['/', '\\']).collect();

    let dir_name = spilt.pop().unwrap();
    let path = spilt.join("/");

    let result = vfs().createdir(&path, dir_name.to_string());
    if result.is_err() {
        println!(
            "failed touching `{dir_name}` in {path}, error: {:?}",
            result.unwrap_err()
        );
    }
}

fn touch(args: Vec<&str>) {
    if args.len() != 2 {
        println!("{}: expected just the new file path", args[0]);
        return;
    }

    let path = get_path(args[1]);

    let mut spilt: Vec<&str> = path.split(['/', '\\']).collect();

    let file_name = spilt.pop().unwrap();
    let path = spilt.join("/");

    let result = vfs().create(&path, file_name.to_string());
    if result.is_err() {
        println!(
            "failed touching `{file_name}` in {path}, error: {:?}",
            result.unwrap_err()
        );
    }
}

fn ls(args: Vec<&str>) {
    if args.len() != 1 {
        println!("{}: expected 0 args", args[0]);
        return;
    }

    let mut dir = vfs().open(&terminal().current_dir).unwrap();
    let files = vfs().readdir(&mut dir).unwrap();
    for file in files {
        println!("{}", file.name());
    }
}

fn cd(args: Vec<&str>) {
    if args.len() != 2 {
        println!("{}: expected only the target directory.", args[0]);
        return;
    }

    let mut path = get_path(args[1]);
    let verify = vfs().verify_path_dir(&path);

    if verify.is_err() {
        println!("{}: path error: {:?}", args[0], verify.unwrap_err())
    } else {
        // must add / because it is stupid, if for example we set the current_dir to ram:/test
        // using `touch` will create an empty file with path ram:/test`file_name`
        // FIXME: consider fixing this next, the code is already spaghetti, the next update should
        // fix all of this
        if !path.ends_with('/') {
            path.push('/');
        }
        terminal().current_dir = path
    }
}

fn cat(args: Vec<&str>) {
    if args.len() != 2 {
        println!("{}: expected only the target file", args[0]);
        return;
    }

    let path = get_path(args[1]);
    let res = vfs().open(&path);

    if res.is_err() {
        println!(
            "{}: failed to open file error: {:?}",
            args[0],
            res.unwrap_err()
        );
    } else {
        let mut opened = res.unwrap();
        let mut buffer: Vec<u8> = Vec::new();
        buffer.resize(opened.size(), 0);

        let read = vfs().read(&mut opened, &mut buffer);
        if read.is_err() {
            println!(
                "{}: failed to read file error: {:?}",
                args[0],
                read.unwrap_err()
            );
            return;
        }

        let output = unsafe { String::from_utf8_unchecked(buffer) };
        println!("{}", output);
    }
}

fn write(args: Vec<&str>) {
    if args.len() != 3 {
        println!("{}: expected the file path then the textual data", args[0]);
        return;
    }

    let path = get_path(args[1]);
    let res = vfs().open(&path);

    if res.is_err() {
        println!(
            "{}: failed to open file error: {:?}",
            args[0],
            res.unwrap_err()
        );
    } else {
        let mut opened = res.unwrap();
        let buffer = args[2].as_bytes();

        let read = vfs().write(&mut opened, &buffer);
        if read.is_err() {
            println!(
                "{}: failed to read file error: {:?}",
                args[0],
                read.unwrap_err()
            );
            return;
        }
    }
}

/// runs `crate::userspace_test as a userspace process`
fn userspace(args: Vec<&str>) {
    if args.len() != 1 {
        println!("{}: no args", args[0]);
        return;
    }

    todo!();
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
        return;
    }

    (match command[0] {
        "echo" => echo,
        "?" | "help" => help,
        "clear" => clear,
        "reboot" => reboot_cmd,
        "shutdown" => shutdown_cmd,

        "plist" => plist,
        "pkill" => pkill,
        "pkillall" => pkillall,

        "ls" => ls,
        "touch" => touch,
        "mkdir" => mkdir,
        "cd" => cd,

        "cat" => cat,
        "write" => write,
        "userspace" => userspace,
        "" => return,
        _ => {
            println!("unknown command {}", command[0]);
            return;
        }
    })(command)
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
        let prompt = (r"\[fg: (0, 255, 0) ||".to_owned() + &terminal().current_dir) + r"||]";

        print!("{} # ", prompt);
        process_command(readln());
    }
}
