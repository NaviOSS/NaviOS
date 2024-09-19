use core::arch::asm;

use alloc::string::ToString;
use bitflags::bitflags;

use crate::{
    debug,
    drivers::vfs::{self, FSResult},
    khalt, scheduler,
    threading::processes::{Process, ProcessStatus},
    utils::elf::{Elf, ElfError},
};

use super::processes::ProcessFlags;

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
    debug!(
        Process,
        "{} waiting for {} to exit ...",
        scheduler().current_process().pid,
        pid
    );

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
    pub struct SpwanFlags: u8 {
        const CLONE_RESOURCES = 1 << 0;
        const CLONE_CWD = 1 << 1;
    }
}

/// FIXME: unsafe because elf_ptr has to be non-null and aligned
/// maybe return an error instead
/// and we need to get rid of the aligned requirment
pub unsafe fn spawn(
    name: &str,
    elf_ptr: *const u8,
    argv: &[&str],
    flags: SpwanFlags,
) -> Result<u64, ElfError> {
    let elf = Elf::new(&*elf_ptr)?;

    let mut process = Process::create(elf.header.entry_point, name, argv, ProcessFlags::USERSPACE)
        .ok()
        .ok_or(ElfError::MapToError)?;
    let pid = process.pid;

    elf.load_exec(&mut *process.root_page_table)?;

    if flags.contains(SpwanFlags::CLONE_RESOURCES) {
        process.resources = scheduler().current_process().resources.clone();
    }

    if flags.contains(SpwanFlags::CLONE_CWD) {
        process.current_dir = scheduler().current_process().current_dir.clone();
    }

    scheduler().add_process(process);
    Ok(pid)
}

/// also ensures the cwd ends with /
/// will only Err if new_dir doesn't exists or is not a directory
#[no_mangle]
pub fn chdir(new_dir: &str) -> FSResult<()> {
    vfs::vfs().verify_path_dir(new_dir)?;
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
