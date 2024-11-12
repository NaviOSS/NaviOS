use core::arch::asm;

use alloc::string::{String, ToString};
use bitflags::bitflags;

use crate::{
    drivers::vfs::{FSResult, VFS_STRUCT},
    khalt,
    memory::paging::allocate_pml4,
    scheduler,
    threading::processes::Process,
    utils::elf::{Elf, ElfError},
};

use super::{
    processes::{ProcessFlags, ProcessInfo, ProcessState},
    resources::Resource,
};

#[no_mangle]
pub fn thread_exit(code: usize) {
    scheduler().current_process().terminate(code, 0);
    // enables interrupts if they were disabled to give control back to the scheduler
    #[cfg(target_arch = "x86_64")]
    unsafe {
        asm!("sti")
    }
    khalt()
}

#[no_mangle]
pub fn thread_yeild() {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        asm!("int 0x20")
    }
}

#[no_mangle]
/// waits for `pid` to exit
/// returns it's exit code after cleaning it up
pub fn wait(pid: u64) -> usize {
    // loops through the processes until it finds the process with `pid` as a zombie
    loop {
        let mut current = scheduler().head.as_mut();
        let mut found = false;

        // cycles through the processes one by one untils it finds the process with `pid`
        // returns the exit code of the process if it's a zombie and cleans it up
        // if it's not a zombie it will be caught by the next above loop
        loop {
            if current
                .next
                .as_ref()
                .is_some_and(|process| process.pid == pid)
            {
                // TODO: rethink returning only the exit code
                // a bit of a hack to fight the borrow checker
                let mut exit_code = None;

                if let ProcessState::Zombie(ref state) = current.next.as_ref().unwrap().state {
                    exit_code = Some(state.exit_code);
                }

                if let Some(exit_code) = exit_code {
                    // cleans up the process
                    current.next = current.next.as_mut().unwrap().next.take();
                    scheduler().processes_count -= 1;
                    return exit_code;
                }

                found = true;
                break;
            }

            if let Some(ref mut process) = current.next {
                current = process;
                thread_yeild()
            } else {
                break;
            }
        }

        if !found {
            return 0;
        }

        thread_yeild();
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    #[repr(C)]
    pub struct SpawnFlags: u8 {
        const CLONE_RESOURCES = 1 << 0;
        const CLONE_CWD = 1 << 1;
    }
}

pub fn spawn(
    name: &str,
    elf_bytes: &[u8],
    argv: &[&str],
    flags: SpawnFlags,
) -> Result<u64, ElfError> {
    let cwd = if flags.contains(SpawnFlags::CLONE_CWD) {
        getcwd().to_string()
    } else {
        String::from("ram:/")
    };

    let elf = Elf::new(elf_bytes)?;

    let mut process = Process::from_elf(elf, name, cwd, argv)?;
    let pid = process.pid;

    let ProcessState::Alive(ref mut state) = process.state else {
        unreachable!()
    };
    // handles the flags
    if flags.contains(SpawnFlags::CLONE_RESOURCES) {
        let clone = scheduler()
            .current_process_state()
            .resource_manager
            .lock()
            .clone_resources();
        state.resource_manager.lock().overwrite_resources(clone);
    }

    scheduler().add_process(process);
    Ok(pid)
}

/// unsafe because function has to be a valid function pointer
/// same as `spawn` but spwans a ring0 function
pub unsafe fn spawn_function(
    name: &str,
    function: usize,
    argv: &[&str],
    flags: SpawnFlags,
) -> Result<u64, ElfError> {
    let page_table_addr = allocate_pml4().map_err(|_| ElfError::MapToError)?;

    let cwd = if flags.contains(SpawnFlags::CLONE_CWD) {
        getcwd().to_string()
    } else {
        String::from("ram:/")
    };

    let mut process = Process::create(
        function,
        name,
        argv,
        0,
        page_table_addr,
        cwd,
        ProcessFlags::empty(),
    )
    .map_err(|_| ElfError::MapToError)?;
    let pid = process.pid;

    let ProcessState::Alive(ref mut state) = process.state else {
        unreachable!()
    };

    if flags.contains(SpawnFlags::CLONE_RESOURCES) {
        let clone = scheduler()
            .current_process_state()
            .resource_manager
            .lock()
            .clone_resources();
        state.resource_manager.lock().overwrite_resources(clone);
    }
    scheduler().add_process(process);
    Ok(pid)
}
/// also ensures the cwd ends with /
/// will only Err if new_dir doesn't exists or is not a directory
#[no_mangle]
pub fn chdir(new_dir: &str) -> FSResult<()> {
    let new_dir = VFS_STRUCT.read().verify_path_dir(new_dir)?;
    let cwd = &mut scheduler().current_process_state().current_dir;
    *cwd = new_dir;
    if !cwd.ends_with('/') {
        cwd.push('/');
    }

    Ok(())
}

#[no_mangle]
pub fn getcwd<'a>() -> &'a str {
    &scheduler().current_process_state().current_dir
}

#[no_mangle]
/// can only Err if pid doesn't belong to process
pub fn pkill(pid: u64) -> Result<(), ()> {
    if pid < scheduler().current_process().pid {
        return Err(());
    }

    let process = scheduler().find(pid).ok_or(())?;
    let current_pid = scheduler().current_process().pid;

    if process.ppid == current_pid || process.pid == current_pid {
        process.terminate(1, current_pid);
        return Ok(());
    }

    // loops through parents and checks if one of the great-grandparents is the current process
    let mut ppid = process.ppid;

    while ppid != 0 {
        let process = scheduler().find(ppid).ok_or(())?;

        if process.pid == current_pid {
            process.terminate(1, current_pid);
            return Ok(());
        }

        ppid = process.ppid;
    }

    Err(())
}

#[no_mangle]
/// collects as much processes as it can in `buffer`
/// collects `buffer.len()` processes
/// if it didn't finish returns Err(())
pub fn pcollect(info: &mut [ProcessInfo]) -> Result<(), ()> {
    let mut current = &mut *scheduler().head;
    let mut i = 1;

    if 0 >= info.len() {
        return Err(());
    }

    info[0] = current.info();

    while let Some(ref mut process) = current.next {
        if i >= info.len() {
            return Err(());
        }

        info[i] = process.info();

        current = &mut *process;
        i += 1;
    }
    Ok(())
}

#[no_mangle]
/// extends program break by `amount`
/// returns the new program break ptr
/// on fail returns null
pub fn sbrk(amount: isize) -> *mut u8 {
    scheduler()
        .current_process_state()
        .extend_data_by(amount)
        .unwrap_or(core::ptr::null_mut())
}
// TODO: lock? or should every resource handle it's own lock?
pub fn get_resource(ri: usize) -> Option<&'static mut Resource> {
    scheduler()
        .current_process_state()
        .resource_manager
        .get_mut()
        .get(ri)
}

pub fn add_resource(resource: Resource) -> usize {
    scheduler()
        .current_process_state()
        .resource_manager
        .lock()
        .add_resource(resource)
}

pub fn remove_resource(ri: usize) -> Result<(), ()> {
    scheduler()
        .current_process_state()
        .resource_manager
        .lock()
        .remove_resource(ri)
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum ErrorStatus {
    None,
    // use when no ErrorStatus is avalible for xyz and you cannot add a new one
    Generic,
    OperationNotSupported,
    // for example an elf class is not supported, there is a difference between NotSupported and
    // OperationNotSupported
    NotSupported,
    // for example a magic value is invaild
    Corrupted,
    InvaildSyscall,
    InvaildResource,
    InvaildPid,
    // instead of panicking syscalls will return this on null and unaligned pointers
    InvaildPtr,
    // for operations that requires a vaild utf8 str...
    InvaildStr,
    InvaildPath,
    NoSuchAFileOrDirectory,
    NotAFile,
    NotADirectory,
    AlreadyExists,
    NotExecutable,
    // would be useful when i add remove related operations to the vfs
    DirectoryNotEmpty,
    // Generic premissions(protection) related error
    MissingPermissions,
    // memory allocations and mapping error, most likely that memory is full
    MMapError,
    Busy,
    // errors sent by processes
    NotEnoughArguments,
}
