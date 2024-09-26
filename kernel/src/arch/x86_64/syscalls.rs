// TODO: figure out errors
// for now errors are a big mess
use core::{
    arch::{asm, global_asm},
    str,
};

use alloc::{slice, string::String};

use crate::{
    drivers::vfs::{self, expose::open},
    threading::{self, expose::SpwanFlags, processes::ProcessInfo},
    utils::{self, expose::SysInfo},
};
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
.set KERNEL_UNSUPPORTED, 7
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
    mov rbp, 0
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
    mov rax, -KERNEL_UNSUPPORTED
    iretq
"
);

extern "x86-interrupt" {
    pub fn syscall_base();
}

macro_rules! sysret {
    ($val: expr) => {
        return $val as u64
    };
}

/// makes a slice from a ptr and len
/// returns INVAILD_PTR_ERR if invaild
macro_rules! make_slice {
    ($ptr: expr, $len: expr) => {
        if !($ptr.is_null() && $len == 0) {
            if $ptr.is_null() || !$ptr.is_aligned() {
                sysret!(INVAILD_PTR_ERR)
            }

            unsafe { slice::from_raw_parts($ptr, $len) }
        } else {
            &[]
        }
    };
}

/// makes a mutable slice from a ptr and len
/// returns INVAILD_PTR_ERR if invaild
macro_rules! make_slice_mut {
    ($ptr: expr, $len: expr) => {
        if !($ptr.is_null() && $len == 0) {
            if $ptr.is_null() || !$ptr.is_aligned() {
                sysret!(INVAILD_PTR_ERR)
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

/// TODO: look more into errors
const INVAILD_PTR_ERR: isize = -257;

#[no_mangle]
extern "C" fn sysopen(path_ptr: *const u8, len: usize) -> u64 {
    let path = make_slice!(path_ptr, len);
    let path = String::from_utf8_lossy(path);

    let ret = match open(&path) {
        Ok(fd) => fd as isize,
        Err(err) => -(err as isize),
    };

    sysret!(ret)
}

#[no_mangle]
extern "C" fn syswrite(fd: usize, ptr: *const u8, len: usize) -> u64 {
    let slice = make_slice!(ptr, len);

    let ret = match vfs::expose::write(fd, slice) {
        Err(err) => -(err as isize),
        Ok(()) => 0,
    };

    sysret!(ret)
}

#[no_mangle]
extern "C" fn sysread(fd: usize, ptr: *mut u8, len: usize) -> u64 {
    let slice = make_slice_mut!(ptr, len);

    let ret = match vfs::expose::read(fd, slice) {
        Err(err) => -(err as isize),
        Ok(bytes_read) => bytes_read as isize,
    };

    sysret!(ret)
}

#[no_mangle]
extern "C" fn sysclose(fd: usize) -> u64 {
    let ret = if let Err(err) = vfs::expose::close(fd) {
        -(err as i16)
    } else {
        0
    };

    sysret!(ret)
}

#[no_mangle]
extern "C" fn syscreate(path_ptr: *const u8, path_len: usize) -> u64 {
    let path = make_slice!(path_ptr, path_len);
    let path = String::from_utf8_lossy(path);

    let ret = if let Err(err) = vfs::expose::create(&path) {
        -(err as i16)
    } else {
        0
    };

    sysret!(ret)
}

#[no_mangle]
extern "C" fn syscreatedir(path_ptr: *const u8, path_len: usize) -> u64 {
    let path = make_slice!(path_ptr, path_len);
    let path = String::from_utf8_lossy(path);

    let ret = if let Err(err) = vfs::expose::createdir(&path) {
        -(err as i16)
    } else {
        0
    };

    sysret!(ret)
}

#[no_mangle]
extern "C" fn sysdiriter_open(dir_ri: usize) -> isize {
    match vfs::expose::diriter_open(dir_ri) {
        Err(err) => -(err as isize),
        Ok(ri) => ri as isize,
    }
}

#[no_mangle]
extern "C" fn sysdiriter_close(diriter_ri: usize) -> isize {
    match vfs::expose::diriter_close(diriter_ri) {
        Err(err) => -(err as isize),
        Ok(()) => 0,
    }
}

#[no_mangle]
unsafe extern "C" fn sysdiriter_next(
    diriter_ri: usize,
    direntry: *mut vfs::expose::DirEntry,
) -> isize {
    if direntry.is_null() {
        return INVAILD_PTR_ERR;
    }

    match vfs::expose::diriter_next(diriter_ri, &mut *direntry) {
        Err(err) => -(err as isize),
        Ok(()) => 0,
    }
}

#[no_mangle]
extern "C" fn syswait(pid: u64) {
    threading::expose::wait(pid);
}

#[no_mangle]
extern "C" fn sysfstat(ri: usize, direntry: &mut vfs::expose::DirEntry) -> isize {
    if let Err(err) = vfs::expose::fstat(ri, direntry) {
        -(err as isize)
    } else {
        0
    }
}

#[no_mangle]
extern "C" fn sysspawn(
    name_ptr: *const u8,
    name_len: usize,
    elf_ptr: *const u8,
    elf_len: usize,
    argc: usize,
    argv: *const (usize, *const u8),
    flags: SpwanFlags,
) -> u64 {
    let name = make_slice!(name_ptr, name_len);
    let name = String::from_utf8_lossy(name);

    let argv = make_slice!(argv, argc);
    let elf_pytes = make_slice!(elf_ptr, elf_len);

    unsafe {
        let argv: &[&str] = core::mem::transmute(argv);
        let ret = match threading::expose::spawn(&name, elf_pytes, argv, flags) {
            Err(err) => -(err as i64),
            Ok(pid) => pid as i64,
        };
        sysret!(ret);
    }
}

#[no_mangle]
extern "C" fn syschdir(path_ptr: *const u8, path_len: usize) -> u64 {
    let slice = make_slice!(path_ptr, path_len);
    let Ok(name) = str::from_utf8(slice) else {
        sysret!(-100i64);
    };

    if let Err(err) = threading::expose::chdir(name) {
        sysret!(-(err as i64))
    }

    sysret!(0)
}

#[no_mangle]
extern "C" fn sysgetcwd(path_ptr: *mut u8, len: usize) -> u64 {
    let slice = make_slice_mut!(path_ptr, len);
    let got = threading::expose::getcwd().as_bytes();

    if got.len() > len {
        return -1i64 as u64;
    }

    slice[..got.len()].copy_from_slice(&got);
    sysret!(got.len());
}

#[no_mangle]
extern "C" fn sysinfo(ptr: *mut SysInfo) -> isize {
    if ptr.is_null() {
        return INVAILD_PTR_ERR;
    }

    unsafe {
        utils::expose::info(&mut *ptr);
    }

    0
}

#[no_mangle]
extern "C" fn syspcollect(ptr: *mut ProcessInfo, len: usize) -> u64 {
    let slice = make_slice_mut!(ptr, len);

    if let Err(()) = threading::expose::pcollect(slice) {
        (-1i64) as u64
    } else {
        0
    }
}

#[no_mangle]
extern "C" fn syssbrk(amount: isize) -> *mut u8 {
    threading::expose::sbrk(amount)
}
