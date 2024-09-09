pub mod framebuffer;
pub mod navitts;

use core::{
    fmt::{self, Write},
    str,
};

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use framebuffer::TerminalMode;

use crate::{
    arch,
    drivers::vfs::{
        self,
        expose::{
            close, create, createdir, diriter_close, diriter_next, diriter_open, open, read,
            DirEntry, FileDescriptorStat,
        },
        vfs,
    },
    globals::terminal,
    kernel, print, println, scheduler, serial,
    threading::{
        processes::{Process, ProcessFlags},
        thread_yeild,
    },
    utils::elf,
    TEST_ELF,
};

#[doc(hidden)]
#[no_mangle]
pub fn _print(args: fmt::Arguments) {
    let mut combined = String::new();
    combined.write_fmt(args).unwrap();

    terminal().write(&combined);
}

pub fn getbyte() -> u8 {
    let old_mode = terminal().mode;
    terminal().mode = TerminalMode::Stdin;

    let last_len = terminal().stdin_buffer.len();
    loop {
        if terminal().stdin_buffer.len() > last_len {
            terminal().mode = old_mode;
            return terminal().stdin_buffer.chars().nth(last_len).unwrap() as u8;
        }

        thread_yeild()
    }
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
        thread_yeild()
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
    write `target_file` `src_text`: writes `src_text` to `target_file`
    userspace: launches test userspace elf
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

    return scheduler().current_process().current_dir.clone() + path;
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

    let result = createdir(&path, dir_name);
    if let Err(err) = result {
        println!("failed touching `{dir_name}` in {path}, error: {:?}", err);
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

    let result = create(&path, file_name);
    if let Err(err) = result {
        println!("failed touching `{file_name}` in {path}, error: {:?}", err);
    }
}

fn ls(args: Vec<&str>) {
    if args.len() != 1 {
        println!("{}: expected 0 args", args[0]);
        return;
    }

    let dir = open(&scheduler().current_process().current_dir).unwrap();
    let diriter = diriter_open(dir).unwrap();

    loop {
        let mut entry = unsafe { DirEntry::zeroed() };
        _ = diriter_next(diriter, &mut entry);

        if entry == unsafe { DirEntry::zeroed() } {
            break;
        }

        let name_string = String::from_utf8(entry.name[..entry.name_length].to_vec()).unwrap();

        println!("{}", name_string);
    }

    _ = diriter_close(diriter);
    close(dir).unwrap();
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
        scheduler().current_process().current_dir = path
    }
}

fn cat(args: Vec<&str>) {
    if args.len() != 2 {
        println!("{}: expected only the target file path", args[0]);
        return;
    }

    let path = get_path(args[1]);
    let res = open(&path);

    if let Err(err) = res {
        println!("{}: failed to open file, error: {:?}", args[0], err);
        return;
    }

    let opened = res.unwrap();
    let mut stat = unsafe { FileDescriptorStat::default() };
    _ = FileDescriptorStat::get(opened, &mut stat);

    let mut buffer: Vec<u8> = Vec::new();
    buffer.resize(stat.size, 0);

    let read = read(opened, &mut buffer);

    if let Err(err) = read {
        println!("{}: failed to read file, error: {:?}", args[0], err);
    }

    let output = unsafe { String::from_utf8_unchecked(buffer) };
    println!("{}", output);

    close(opened).unwrap();
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

/// runs `crate::userspace_test as a userspace process`
fn userspace(args: Vec<&str>) {
    if args.len() != 1 {
        println!("{}: excepts no args", args[0]);
        return;
    }

    let elf_bytes = TEST_ELF.to_vec();
    let elf = elf::Elf::new(&elf_bytes[0]).unwrap();

    let process = Process::create(elf.header.entry_point, "user_test", ProcessFlags::USERSPACE);
    unsafe {
        elf.load_exec(&mut *process.root_page_table).unwrap();
    }

    scheduler().add_process(process);
}

fn meminfo(args: Vec<&str>) {
    if args.len() != 1 {
        println!("{}: excepts no args", args[0]);
        return;
    }

    let bitmap = &*kernel().frame_allocator().bitmap;
    let mut memory_max = 0;
    let mut memory_used = 0;
    let mut memory_ava = 0;

    for byte in bitmap {
        for i in 0..8 {
            memory_max += crate::memory::paging::PAGE_SIZE;
            let frame_used = (byte >> i) & 1 == 1;
            if frame_used {
                memory_used += crate::memory::paging::PAGE_SIZE;
            } else {
                memory_ava += crate::memory::paging::PAGE_SIZE;
            }
        }
    }

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
        "meminfo" => meminfo,
        "breakpoint" => breakpoint,
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
        print!(
            r"\[fg: (0, 255, 0) ||{}||] # ",
            scheduler().current_process().current_dir
        );
        process_command(readln());
    }
}
