#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(asm_const)]
use core::arch::global_asm;

const MULTIBOOT2_HEADER_MAGIC: u32 = 0x1BADB002;
const MULTIBOOT2_HEADER_FLAGS: u32 = 0; // Set the appropriate flags for your setup
const MULTIBOOT2_HEADER_CHECKSUM: i32 =
    0 - MULTIBOOT2_HEADER_MAGIC as i32 - MULTIBOOT2_HEADER_FLAGS as i32;

global_asm!(
    "
    .section .multiboot
    .align 4
    .long   {MAGIC}                // magic number
    .long   0                         // flags
    .long   {CHECKSUM}                // checksum

    // // stack
    // .section .bss
    // .align 16
    // stack_bottom:
    // .skip 16384
    // stack_top:

    .section .text
    .global _start
    _start:
        mov esp, {STACK_TOP}
        call kmain
    ",
    MAGIC = const MULTIBOOT2_HEADER_MAGIC,
    CHECKSUM = const MULTIBOOT2_HEADER_CHECKSUM,
    STACK_TOP = const 0x300000
);

mod kernel;
use core::arch::asm;

macro_rules! s {
    ($str: expr) => {
        $str.as_ptr()
    };
}

use core::panic::PanicInfo;

use kernel::vga;
use kernel::{
    gdt::{self, GDTType, GDT},
    vga::{kerr, kput, kwrite},
};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kput(b'\n'); // ensures correctly formated panic

    kerr(s!(b"Kernel panic: \0"));
    kerr(info.message().as_str().unwrap().as_ptr());
    kerr(s!(b"\ncannot continue execution kernel will now hang\0\n"));
    loop {}
}

pub fn strlen(cstr: *const u8) -> usize {
    let mut len = 0;

    while unsafe { *cstr.offset(len as isize) } != b'\0' {
        len += 1;
    }
    len
}

// gpt4 generated
pub fn u32_to_hex_array(value: u32) -> [u8; 11] {
    let mut hex_array = [0u8; 11];
    hex_array[0] = b'0';
    hex_array[1] = b'x';

    for i in 0..8 {
        let nibble = (value >> (28 - i * 4)) & 0xF;
        hex_array[i + 2] = match nibble {
            0..=9 => b'0' + nibble as u8,
            _ => b'a' + (nibble - 10) as u8,
        };
    }

    hex_array[10] = b'\0';

    hex_array
}

fn write_hex(hex: u32) {
    kwrite(u32_to_hex_array(hex).as_ptr());
    kput(b'\n');
}

#[inline]
fn print_registers() {
    let (ds, es, fs, gs, ss, cs, eax): (u16, u16, u16, u16, u16, u16, u32);

    unsafe {
        asm!(
            "
        mov {0:x}, ds
        mov {1:x}, es
        mov {2:x}, fs
        mov {3:x}, gs
        mov {4:x}, ss
        mov {5:x}, cs

        "
            , out(reg) ds
            , out(reg) es
            , out(reg) fs
            , out(reg) gs
            , out(reg) ss
            , out(reg) cs
        );
    }

    kwrite(s!("ds: \0"));
    write_hex(ds.into());

    kwrite(s!("es: \0"));
    write_hex(es.into());

    kwrite(s!("fs: \0"));
    write_hex(fs.into());

    kwrite(s!("gs: \0"));
    write_hex(gs.into());

    kwrite(s!("ss: \0"));
    write_hex(ss.into());

    kwrite(s!("cs: \0"));
    write_hex(cs.into());

    unsafe {
        asm!(
            "
            mov eax, 0xFFFFFFFF
            mov {0}, eax
            ",
            out(reg) eax,
        )
    }

    kwrite(s!("not xor eax: \0"));
    write_hex(eax);
}

pub extern "C" fn kinit() {
    vga::init_vga();
    kwrite(s!("disabling interrupts....\n\0"));
    unsafe {
        asm!(
            "
            cli
            ",
            options(nostack)
        );
    };
    kwrite(s!("initing the gdt....\n\0"));
    gdt::init_gdt();
    // entering protected mode
    unsafe {
        asm!(
            "
            mov eax, cr0
            or al, 1
            mov cr0, eax
            "
        );
    };
    kwrite(s!("initing pm....\n\0"));
    gdt::init_pm();

    kwrite(s!("init done\0\n"));
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    kinit();

    print_registers();

    write_hex(gdt::GDT_DESCRIPTOR.limit as u32);
    write_hex(size_of::<gdt::GDTType>() as u32);
    write_hex(gdt::GDT_DESCRIPTOR.base);
    write_hex(&*GDT as *const GDTType as u32);

    kwrite(s!(b"Hello, world!\n\0"));

    loop {}
}
