pub mod device;
pub mod framebuffer;
pub mod navitts;

use core::{
    fmt::{self, Write},
    str,
};

use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
    vec::Vec,
};
use framebuffer::TerminalMode;

use crate::{
    arch,
    drivers::vfs::{
        self,
        expose::{close, open, DirEntry},
        FSError, FSResult, InodeType,
    },
    globals::terminal,
    print, println, scheduler, serial,
    threading::{self, expose::SpwanFlags, processes::ProcessInfo},
    utils::{self, expose::SysInfo},
};

#[doc(hidden)]
#[no_mangle]
pub fn _print(args: fmt::Arguments) {
    let mut combined = String::new();
    combined.write_fmt(args).unwrap();

    terminal().write(&combined);
}

/// TODO: replace with a normal read?
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
        threading::expose::thread_yeild()
    }
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
commands (additionally there may be some elfs in sys:/programs/ which were not listed here):
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
    write `target_file` `src_text`: writes `src_text` to `target_file`
    test: launches test userspace elf located at sys:/programs/
    meminfo: gives some memory info
    breakpoint: executes int3"
    );
}

fn clear(args: Vec<&str>) {
    if args.len() != 1 {
        println!("{}: expected 0 args", args[0]);
        return;
    }

    print!("you sure? y\\N: ");
    let confirm = readln();
    let viewport = &mut framebuffer::VIEWPORT.lock();
    if confirm.to_uppercase() == "Y" {
        terminal().clear(viewport)
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

    let mut info = SysInfo::null();
    utils::expose::info(&mut info);

    let mut process_list: Vec<ProcessInfo> = Vec::with_capacity(info.processes_count);
    process_list.resize(info.processes_count, ProcessInfo::null());
    threading::expose::pcollect(&mut process_list).unwrap();

    for ProcessInfo {
        ppid,
        pid,
        name,
        status: _,
    } in process_list
    {
        let mut name = name.to_vec();
        while name.last() == Some(&0) {
            name.pop();
        }

        println!("{}:  {}  {}", str::from_utf8(&name).unwrap(), pid, ppid);
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

    threading::expose::pkill(pid).unwrap_or_else(|_| {
        println!(
            "couldn't find a process with pid `{}` or the current process doesn't own it",
            pid
        )
    });
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

    return threading::expose::getcwd().to_owned() + path;
}

fn cd(args: Vec<&str>) {
    if args.len() != 2 {
        println!("{}: expected only the target directory.", args[0]);
        return;
    }

    let path = get_path(args[1]);
    if let Err(err) = threading::expose::chdir(&path) {
        println!("{}: path error: {:?}", args[0], err)
    }
}

fn write(args: Vec<&str>) {
    if args.len() != 3 {
        println!("{}: expected the file path then the textual data", args[0]);
        return;
    }

    let path = get_path(args[1]);
    let res = open(&path);

    if let Err(err) = res {
        println!("{}: failed to open file, error: {:?}", args[0], err);
        return;
    }

    let opened = res.unwrap();
    let buffer = args[2].as_bytes();

    let wrote = vfs::expose::write(opened, &buffer);
    if let Err(err) = wrote {
        println!("{}: failed to write to file, error: {:?}", args[0], err);
    }

    close(opened).unwrap();
}

fn meminfo(args: Vec<&str>) {
    if args.len() != 1 {
        println!("{}: excepts no args", args[0]);
        return;
    }

    let mut info = SysInfo::null();
    utils::expose::info(&mut info);

    let (memory_max, memory_used) = (info.total_mem, info.used_mem);
    let memory_ava = memory_max - memory_used;

    println!("memory info:");
    println!(
        "memory_max: {}    memory_used: {}\nmemory_ava: {}",
        memory_max, memory_used, memory_ava
    );

    println!(
        "{}KiBs out of {}KiBs used",
        memory_used / 1024,
        memory_max / 1024
    );

    println!(
        "{}MiBs out of {}MiBs used",
        memory_used / 1024 / 1024,
        memory_max / 1024 / 1024
    );

    println!("note that this is not 100% accurate, memory_max and memory_used is more then the actual number in 90% of cases, unusable memory also counts as used")
}

fn breakpoint(args: Vec<&str>) {
    if args.len() != 1 {
        println!("{}: excepts no args", args[0]);
        return;
    }

    unsafe { core::arch::asm!("int3") }
}

/// lookups command in PATH and cwd and spwans and waits for it if it exists
fn execute_command(args: Vec<&str>) -> FSResult<()> {
    let command = args[0];
    let path_var = &[threading::expose::getcwd(), "sys:/programs/"];

    for cwd_path in path_var {
        let cwd_path = cwd_path.to_string();

        let cwd = vfs::expose::open(&cwd_path)?;
        let diriter = vfs::expose::diriter_open(cwd)?;

        let mut entry = unsafe { DirEntry::zeroed() };
        loop {
            vfs::expose::diriter_next(diriter, &mut entry)?;
            if entry == unsafe { DirEntry::zeroed() } {
                break;
            }

            if entry.name() == command && entry.kind == InodeType::File {
                let full_path = cwd_path + command;
                let opened = vfs::expose::open(&full_path)?;

                let mut buffer = Vec::with_capacity(entry.size);
                buffer.resize(entry.size, 0);
                vfs::expose::read(opened, &mut buffer)?;

                vfs::expose::close(opened)?;

                // FIXME: should be CLONE_RESOURCES tho
                let pid = unsafe {
                    threading::expose::spawn(command, &buffer, &args, SpwanFlags::CLONE_CWD)
                        .ok()
                        .ok_or(FSError::NotExecuteable)?
                };

                threading::expose::wait(pid);

                vfs::expose::diriter_close(diriter)?;
                vfs::expose::close(cwd)?;
                return Ok(());
            }
        }

        vfs::expose::diriter_close(diriter)?;
        vfs::expose::close(cwd)?;
    }
    Err(FSError::NoSuchAFileOrDirectory)
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
        "?" | "help" => help,
        "clear" => clear,
        "reboot" => reboot_cmd,
        "shutdown" => shutdown_cmd,

        "plist" => plist,
        "pkill" => pkill,
        "pkillall" => pkillall,

        "cd" => cd,

        "write" => write,
        "meminfo" => meminfo,
        "breakpoint" => breakpoint,
        "page_fault" => unsafe {
            *(0xdeadbeef as *mut u8) = 0xAA;
            unreachable!()
        },
        "" => return,
        cmd => {
            if let Err(err) = execute_command(command) {
                println!("unknown command {}: {:?}", cmd, err);
            }

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
    print!("\\[fg: (255, 255, 255) ||\nwelcome to NaviOS!\ntype help or ? for a list of avalible commands\nyou are now in ram:/ a playground, sys: is also mounted it contains the init ramdisk\n||]");

    loop {
        print!(r"\[fg: (0, 255, 0) ||{}||] # ", threading::expose::getcwd());
        process_command(readln());
    }
}
