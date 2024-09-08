use core::{
    arch::{asm, global_asm},
    isize,
};

use alloc::{slice, string::String};

use crate::{
    drivers::vfs::{self, expose::open},
    print,
    terminal::{self, framebuffer::VIEWPORT},
    threading::{thread_exit, thread_yeild},
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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// registers pushed by syscall_base
struct SyscallRegisters {
    _r15: u64,
    _r14: u64,
    _r13: u64,
    _r12: u64,
    _r11: u64,
    _r10: u64,
    _r9: u64,
    _r8: u64,
    _rbp: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
}

macro_rules! sysret {
    ($val: expr) => {
        unsafe {
            asm!("mov rax, {:r}", in(reg) $val, options(nostack));
            return;
        }
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
extern "C" fn sysopen(registers: SyscallRegisters) {
    let path_ptr = registers.rdi as *const u8;
    let len = registers.rsi as usize;

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
extern "C" fn syswrite(registers: SyscallRegisters) {
    let fd = registers.rdi as usize;
    let ptr = registers.rsi as *const u8;

    if ptr.is_null() || !ptr.is_aligned() {
        sysret!(INVAILD_PTR_ERR)
    }

    let len = registers.rdx as usize;

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
        fd => {
            if let Err(err) = vfs::expose::write(fd, slice) {
                -(err as i16)
            } else {
                0
            }
        }
    };

    sysret!(ret)
}

#[no_mangle]
extern "C" fn sysread(registers: SyscallRegisters) {
    let fd = registers.rdi as usize;
    let ptr = registers.rsi as *mut u8;

    if ptr.is_null() || !ptr.is_aligned() {
        sysret!(INVAILD_PTR_ERR)
    }

    let len = registers.rdx as usize;

    let slice = unsafe { slice::from_raw_parts_mut(ptr, len) };

    let ret = match fd {
        0 => {
            for i in 0..slice.len() {
                slice[i] = terminal::getbyte();
            }
            // flushing stdin
            crate::terminal().stdin_buffer.clear();
            0
        }
        _ => {
            if let Err(err) = vfs::expose::read(fd, slice) {
                -(err as i16)
            } else {
                0
            }
        }
    };

    sysret!(ret)
}
