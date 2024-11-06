// TODO: figure out errors
// for now errors are a big mess
use core::{
    arch::{asm, global_asm},
    str,
};

use alloc::{slice, string::String};

use crate::{
    drivers::vfs::{self, expose::open, FSError},
    threading::{
        self,
        expose::{ErrorStatus, SpawnFlags},
        processes::ProcessInfo,
    },
    utils::{self, expose::SysInfo},
};

use super::interrupts::InterruptFrame;
/// used sometimes for debugging syscalls
#[allow(dead_code)]
#[derive(Debug, Clone)]
#[repr(C)]
pub struct SyscallContext {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rbp: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub frame: InterruptFrame,
}
global_asm!(
    "
.section .rodata
syscall_table:
    .quad sysexit
    .quad sysyield
    .quad sysopen
    .quad syswrite
    .quad sysread
    .quad sysclose
    .quad syscreate
    .quad syscreatedir
    .quad sysdiriter_open
    .quad sysdiriter_close
    .quad sysdiriter_next
    .quad syswait
    .quad sysfstat
    .quad sysspawn
    .quad syschdir
    .quad sysgetcwd
    .quad sysinfo
    .quad syspcollect
    .quad syssbrk
syscall_table_end:

SYSCALL_TABLE_INFO:
    .quad (syscall_table_end - syscall_table) / 8
.section .text
.global syscall_base

syscall_base:
    cmp rax, [SYSCALL_TABLE_INFO]
    jge unsupported
    push rbx
    push rcx
    push rdx
    push rsi
    push rdi
    push rbp
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15
    call [syscall_table + rax * 8]
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rbp
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rbx
    iretq
unsupported:
    mov rax, {0}
    iretq
", const ErrorStatus::InvaildSyscall as u64
);

extern "x86-interrupt" {
    pub fn syscall_base();
}

/// makes a slice from a ptr and len
/// returns ErrorStatus::InvaildPtr if invaild
macro_rules! make_slice {
    ($ptr: expr, $len: expr) => {
        if !($ptr.is_null() && $len == 0) {
            if $ptr.is_null() || !$ptr.is_aligned() {
                return ErrorStatus::InvaildPtr;
            }

            unsafe { slice::from_raw_parts($ptr, $len) }
        } else {
            &[]
        }
    };
}

/// makes a mutable slice from a ptr and len
/// returns ErrorStatus::InvaildPtr if invaild
macro_rules! make_slice_mut {
    ($ptr: expr, $len: expr) => {
        if !($ptr.is_null() && $len == 0) {
            if $ptr.is_null() || !$ptr.is_aligned() {
                return ErrorStatus::InvaildPtr;
            }

            unsafe { slice::from_raw_parts_mut($ptr, $len) }
        } else {
            &mut []
        }
    };
}
/// for now
#[no_mangle]
extern "C" fn sysexit() {
    unsafe { asm!("sti") }
    threading::expose::thread_exit();
}

#[no_mangle]
extern "C" fn sysyield() {
    threading::expose::thread_yeild()
}

#[no_mangle]
extern "C" fn sysopen(path_ptr: *const u8, len: usize, dest_fd: *mut usize) -> ErrorStatus {
    if dest_fd.is_null() {
        return ErrorStatus::InvaildPtr;
    }

    let path = make_slice!(path_ptr, len);
    let path = String::from_utf8_lossy(path);

    match open(&path) {
        Ok(fd) => unsafe {
            *dest_fd = fd;
            ErrorStatus::None
        },
        Err(err) => err.into(),
    }
}

#[no_mangle]
extern "C" fn syswrite(fd: usize, ptr: *const u8, len: usize) -> ErrorStatus {
    let slice = make_slice!(ptr, len);
    while let Err(err) = vfs::expose::write(fd, slice) {
        match err {
            FSError::ResourceBusy => {
                threading::expose::thread_yeild();
            }
            _ => return err.into(),
        }
    }
    ErrorStatus::None
}

#[no_mangle]
extern "C" fn sysread(fd: usize, ptr: *mut u8, len: usize, dest_read: *mut usize) -> ErrorStatus {
    let slice = make_slice_mut!(ptr, len);

    loop {
        match vfs::expose::read(fd, slice) {
            Err(FSError::ResourceBusy) => threading::expose::thread_yeild(),
            Err(err) => return err.into(),
            Ok(bytes_read) => unsafe {
                *dest_read = bytes_read;
                return ErrorStatus::None;
            },
        }
    }
}

#[no_mangle]
extern "C" fn sysclose(fd: usize) -> ErrorStatus {
    if let Err(err) = vfs::expose::close(fd) {
        err.into()
    } else {
        ErrorStatus::None
    }
}

#[no_mangle]
extern "C" fn syscreate(path_ptr: *const u8, path_len: usize) -> ErrorStatus {
    let path = make_slice!(path_ptr, path_len);
    let path = String::from_utf8_lossy(path);

    if let Err(err) = vfs::expose::create(&path) {
        err.into()
    } else {
        ErrorStatus::None
    }
}

#[no_mangle]
extern "C" fn syscreatedir(path_ptr: *const u8, path_len: usize) -> ErrorStatus {
    let path = make_slice!(path_ptr, path_len);
    let path = String::from_utf8_lossy(path);

    if let Err(err) = vfs::expose::createdir(&path) {
        err.into()
    } else {
        ErrorStatus::None
    }
}

#[no_mangle]
extern "C" fn sysdiriter_open(dir_ri: usize, dest_diriter: *mut usize) -> ErrorStatus {
    match vfs::expose::diriter_open(dir_ri) {
        Err(err) => err.into(),
        Ok(ri) => unsafe {
            *dest_diriter = ri;
            ErrorStatus::None
        },
    }
}

#[no_mangle]
extern "C" fn sysdiriter_close(diriter_ri: usize) -> ErrorStatus {
    match vfs::expose::diriter_close(diriter_ri) {
        Err(err) => err.into(),
        Ok(()) => ErrorStatus::None,
    }
}

#[no_mangle]
unsafe extern "C" fn sysdiriter_next(
    diriter_ri: usize,
    direntry: *mut vfs::expose::DirEntry,
) -> ErrorStatus {
    if direntry.is_null() || !direntry.is_aligned() {
        return ErrorStatus::InvaildPtr;
    }

    match vfs::expose::diriter_next(diriter_ri, &mut *direntry) {
        Err(err) => err.into(),
        Ok(()) => ErrorStatus::None,
    }
}

#[no_mangle]
extern "C" fn syswait(pid: u64) {
    threading::expose::wait(pid);
}

#[no_mangle]
extern "C" fn sysfstat(ri: usize, direntry: *mut vfs::expose::DirEntry) -> ErrorStatus {
    if direntry.is_null() || !direntry.is_aligned() {
        return ErrorStatus::InvaildPtr;
    }

    unsafe {
        if let Err(err) = vfs::expose::fstat(ri, &mut *direntry) {
            err.into()
        } else {
            ErrorStatus::None
        }
    }
}
// argv can be null
// name can be null but the spawned process name will be different in case of spawn or pspawn
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SpawnConfig {
    pub name_ptr: *const u8,
    pub name_len: usize,
    pub argv: *mut (*const u8, usize),
    pub argc: usize,
    pub flags: SpawnFlags,
}

// if dest_pid is null we will just ignore it
#[no_mangle]
extern "C" fn sysspawn(
    elf_ptr: *const u8,
    elf_len: usize,
    config: *const SpawnConfig,
    dest_pid: *mut u64,
) -> ErrorStatus {
    let (name_ptr, name_len, argc, argv, flags) = unsafe {
        let config = *config;
        (
            config.name_ptr,
            config.name_len,
            config.argc,
            config.argv,
            config.flags,
        )
    };

    let name = if !name_ptr.is_null() {
        make_slice!(name_ptr, name_len)
    } else {
        &[]
    };
    let name = String::from_utf8_lossy(name);

    let argv = if !argv.is_null() {
        make_slice_mut!(argv, argc)
    } else {
        &mut []
    };

    let argv_str: &mut [&str] = unsafe { core::mem::transmute(&mut *argv) };

    for (i, arg) in argv.iter().enumerate() {
        // transmut doesn't work we make it work here
        let slice = make_slice!(arg.0, arg.1);
        unsafe {
            argv_str[i] = str::from_utf8_unchecked(slice);
        }
        // argv[i] is invaild after this
        // argv_str[i] is argv[i] but in a rusty way
    }

    let elf_bytes = make_slice!(elf_ptr, elf_len);
    unsafe {
        match threading::expose::spawn(&name, elf_bytes, argv_str, flags) {
            Err(err) => err.into(),
            Ok(pid) => {
                if !dest_pid.is_null() {
                    *dest_pid = pid
                }
                ErrorStatus::None
            }
        }
    }
}

#[no_mangle]
extern "C" fn syschdir(path_ptr: *const u8, path_len: usize) -> ErrorStatus {
    let slice = make_slice!(path_ptr, path_len);
    let Ok(name) = str::from_utf8(slice) else {
        return ErrorStatus::InvaildStr;
    };

    if let Err(err) = threading::expose::chdir(name) {
        err.into()
    } else {
        ErrorStatus::None
    }
}

#[no_mangle]
extern "C" fn sysgetcwd(path_ptr: *mut u8, len: usize, dest_len: *mut usize) -> ErrorStatus {
    let slice = make_slice_mut!(path_ptr, len);
    let got = threading::expose::getcwd().as_bytes();

    if got.len() > len {
        return ErrorStatus::Generic;
    }

    slice[..got.len()].copy_from_slice(&got);

    if !dest_len.is_null() {
        unsafe {
            *dest_len = got.len();
        }
    }

    ErrorStatus::None
}

#[no_mangle]
extern "C" fn sysinfo(ptr: *mut SysInfo) -> ErrorStatus {
    if ptr.is_null() || !ptr.is_aligned() {
        return ErrorStatus::InvaildPtr;
    }

    unsafe {
        utils::expose::info(&mut *ptr);
    }

    ErrorStatus::None
}

#[no_mangle]
extern "C" fn syspcollect(ptr: *mut ProcessInfo, len: usize) -> ErrorStatus {
    let slice = make_slice_mut!(ptr, len);

    if let Err(()) = threading::expose::pcollect(slice) {
        ErrorStatus::Generic
    } else {
        ErrorStatus::None
    }
}

// on fail returns null for unknown reasons
#[no_mangle]
extern "C" fn syssbrk(amount: isize) -> *mut u8 {
    threading::expose::sbrk(amount)
}
