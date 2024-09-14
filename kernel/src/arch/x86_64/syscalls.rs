use core::arch::{asm, global_asm};

use alloc::{slice, string::String};

use crate::{
    drivers::vfs::{self, expose::open},
    print,
    terminal::framebuffer::VIEWPORT,
    threading::{thread_exit, thread_yeild, wait},
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
    .quad 0 // FIXME: replace with diriter_open
    .quad 0 // FIXME: replace with diriter_close
    .quad 0 // FIXME: replace with diriter_next
    .quad syswait
syscall_table_end:

SYSCALL_TABLE_INFO:
    .word (syscall_table_end - syscall_table) / 8
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

/// for now
#[no_mangle]
extern "C" fn sysexit() {
    unsafe { asm!("sti") }
    thread_exit();
}

#[no_mangle]
extern "C" fn sysyield() {
    thread_yeild()
}

/// TODO: look more into errors
const INVAILD_PTR_ERR: isize = -257;

#[no_mangle]
extern "C" fn sysopen(path_ptr: *const u8, len: usize) -> u64 {
    if path_ptr.is_null() || !path_ptr.is_aligned() {
        sysret!(INVAILD_PTR_ERR)
    }

    let path = unsafe { slice::from_raw_parts(path_ptr, len) };
    let path = String::from_utf8_lossy(path);

    let ret = match open(&path) {
        Ok(fd) => fd as isize,
        Err(err) => -(err as isize),
    };

    sysret!(ret)
}

#[no_mangle]
extern "C" fn syswrite(fd: usize, ptr: *const u8, len: usize) -> u64 {
    if ptr.is_null() || !ptr.is_aligned() {
        sysret!(INVAILD_PTR_ERR)
    }

    let slice = unsafe { slice::from_raw_parts(ptr, len) };
    let ret = match fd {
        1 => {
            let str = String::from_utf8_lossy(slice);
            while VIEWPORT.is_locked() {
                thread_yeild()
            }

            print!("{}", str);
            0
        }
        fd => match vfs::expose::write(fd, slice) {
            Err(err) => -(err as isize),
            Ok(()) => 0,
        },
    };

    sysret!(ret)
}

#[no_mangle]
extern "C" fn sysread(fd: usize, ptr: *mut u8, len: usize) -> u64 {
    if ptr.is_null() || !ptr.is_aligned() {
        sysret!(INVAILD_PTR_ERR)
    }

    let slice = unsafe { slice::from_raw_parts_mut(ptr, len) };

    let ret = match fd {
        // 0 => {
        //     for i in 0..slice.len() {
        //         slice[i] = terminal::getbyte();
        //     }
        //     // flushing stdin
        //     crate::terminal().stdin_buffer.clear();
        //     0
        // }
        _ => match vfs::expose::read(fd, slice) {
            Err(err) => -(err as isize),
            Ok(bytes_read) => bytes_read as isize,
        },
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
extern "C" fn syscreate(
    path_ptr: *const u8,
    path_len: usize,
    name_ptr: *const u8,
    name_len: usize,
) -> u64 {
    let path = unsafe { slice::from_raw_parts(path_ptr, path_len) };
    let path = String::from_utf8_lossy(path);

    let name = unsafe { slice::from_raw_parts(name_ptr, name_len) };
    let name = String::from_utf8_lossy(name);

    let ret = if let Err(err) = vfs::expose::create(&path, &name) {
        -(err as i16)
    } else {
        0
    };

    sysret!(ret)
}

#[no_mangle]
extern "C" fn syscreatedir(
    path_ptr: *const u8,
    path_len: usize,
    name_ptr: *const u8,
    name_len: usize,
) -> u64 {
    let path = unsafe { slice::from_raw_parts(path_ptr, path_len) };
    let path = String::from_utf8_lossy(path);

    let name = unsafe { slice::from_raw_parts(name_ptr, name_len) };
    let name = String::from_utf8_lossy(name);

    let ret = if let Err(err) = vfs::expose::createdir(&path, &name) {
        -(err as i16)
    } else {
        0
    };

    sysret!(ret)
}

#[no_mangle]
extern "C" fn syswait(pid: u64) {
    wait(pid);
}
