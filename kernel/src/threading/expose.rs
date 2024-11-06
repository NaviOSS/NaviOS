use core::arch::asm;

use alloc::string::ToString;
use bitflags::bitflags;

use crate::{
    drivers::vfs::{FSResult, VFS_STRUCT},
    khalt, scheduler,
    threading::processes::{Process, ProcessStatus},
    utils::elf::{Elf, ElfError},
};

use super::processes::{ProcessFlags, ProcessInfo};

#[no_mangle]
pub fn thread_exit() {
    scheduler().current_process().status = ProcessStatus::WaitingForBurying;
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
pub fn wait(pid: u64) {
    // debug!(
    //     Process,
    //     "{} waiting for {} to exit ...",
    //     scheduler().current_process().pid,
    //     pid
    // );

    loop {
        let mut current = scheduler().head.as_mut();
        let mut found = false;

        while let Some(ref mut process) = current.next {
            if process.pid == pid {
                found = true;
                if process.status == ProcessStatus::WaitingForBurying {
                    return;
                }
            }

            current = process;
            thread_yeild()
        }

        if !found {
            return;
        }

        thread_yeild()
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

/// FIXME: unsafe because elf_ptr has to be non-null and aligned
/// maybe return an error instead
/// and we need to get rid of the aligned requirment
pub unsafe fn spawn(
    name: &str,
    elf_bytes: &[u8],
    argv: &[&str],
    flags: SpawnFlags,
) -> Result<u64, ElfError> {
    let elf = Elf::new(elf_bytes)?;

    let mut process = Process::from_elf(elf, name, argv)?;
    let pid = process.pid;

    if flags.contains(SpawnFlags::CLONE_RESOURCES) {
        process.resources = scheduler().current_process().resources.clone();
    }

    if flags.contains(SpawnFlags::CLONE_CWD) {
        process.current_dir = scheduler().current_process().current_dir.clone();
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
    let mut process = Process::create(function, name, argv, ProcessFlags::empty())
        .map_err(|_| ElfError::MapToError)?;
    let pid = process.pid;

    if flags.contains(SpawnFlags::CLONE_RESOURCES) {
        process.resources = scheduler().current_process().resources.clone();
    }

    if flags.contains(SpawnFlags::CLONE_CWD) {
        process.current_dir = scheduler().current_process().current_dir.clone();
    }

    scheduler().add_process(process);
    Ok(pid)
}
/// also ensures the cwd ends with /
/// will only Err if new_dir doesn't exists or is not a directory
#[no_mangle]
pub fn chdir(new_dir: &str) -> FSResult<()> {
    VFS_STRUCT.read().verify_path_dir(new_dir)?;
    let cwd = &mut scheduler().current_process().current_dir;
    *cwd = new_dir.to_string();
    if !cwd.ends_with('/') {
        cwd.push('/');
    }

    Ok(())
}

#[no_mangle]
pub fn getcwd<'a>() -> &'a str {
    &scheduler().current_process().current_dir
}

#[no_mangle]
/// can only Err if pid doesn't belong to process
pub fn pkill(pid: u64) -> Result<(), ()> {
    if pid < scheduler().current_process().pid {
        return Err(());
    }

    let process = scheduler().find(pid).ok_or(())?;

    if process.ppid == scheduler().current_process().pid
        || process.pid == scheduler().current_process().pid
    {
        process.status = ProcessStatus::WaitingForBurying;
        return Ok(());
    }

    let mut ppid = process.ppid;

    while ppid != 0 {
        let process = scheduler().find(ppid).ok_or(())?;

        if process.pid == scheduler().current_process().pid {
            process.status = ProcessStatus::WaitingForBurying;
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
        .current_process()
        .extend_data_by(amount)
        .unwrap_or(core::ptr::null_mut())
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
    InvaildDrive,
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
