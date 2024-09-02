#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]
#![feature(custom_test_frameworks)]
#[cfg(feature = "test")]
mod test;

mod arch;
mod drivers;
mod globals;
mod limine;
mod memory;
mod terminal;
mod threading;
mod utils;

extern crate alloc;
use arch::threading::restore_cpu_status;
use arch::x86_64::serial;

use drivers::keyboard::Key;
use drivers::vfs;
use globals::*;

use limine::get_phy_offset;
use limine::get_phy_offset_end;
use limine::MEMORY_SIZE;
use memory::frame_allocator::RegionAllocator;
pub use memory::PhysAddr;
pub use memory::VirtAddr;
use terminal::framebuffer::Terminal;
use threading::ProcessFlags;
use threading::Scheduler;

const TEST_ELF: &[u8] = include_bytes!("../../user/test");

#[macro_export]
macro_rules! print {
   ($($arg:tt)*) => ($crate::terminal::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => (crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! serial {
    ($($arg:tt)*) => {
        crate::arch::x86_64::serial::_serial(format_args!($($arg)*))
    };
}

use core::arch::asm;
#[no_mangle]
#[inline]
pub fn khalt() -> ! {
    loop {
        unsafe { asm!("hlt") }
    }
}

#[allow(unused_imports)]
use core::panic::PanicInfo;

/// prints to both the serial and the terminal doesn't print to the terminal if it panicked or if
/// it is not ready...
#[macro_export]
macro_rules! cross_println {
    ($($arg:tt)*) => {
        serial!($($arg)*);
        serial!("\n");

        if terminal_inited() && !terminal().panicked {
            terminal().panicked = true;

            println!($($arg)*);

            terminal().panicked = false;
        }
    };
}

#[allow(dead_code)]
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe { asm!("cli") }
    cross_println!(
        "kernel panic:\n{}, at {}",
        info.message(),
        info.location().unwrap()
    );
    print_stack_trace();

    khalt()
}

#[allow(unused)]
fn print_stack_trace() {
    let mut fp: usize;

    unsafe {
        core::arch::asm!("mov {}, rbp", out(reg) fp);

        cross_println!("stack trace: ");
        while fp != 0 {
            let return_address = *(fp as *const usize).offset(1);
            let name = {
                if kernel_inited() {
                    let sym = kernel().elf.sym_from_value_range(return_address);

                    if sym.is_some() {
                        kernel().elf.string_table_index(sym.unwrap().name_index)
                    } else {
                        "??"
                    }
                } else {
                    "??"
                }
            };

            cross_println!("  {:#x} <{}>", return_address, name);
            fp = *(fp as *const usize);
        }
    }
}

#[no_mangle]
pub extern "C" fn kinit() {
    // initing globals
    let phy_offset = get_phy_offset();
    let kernel_img = limine::kernel_image_info();

    serial!(
        "image at: 0x{:x}\nlen: 0x{:x}\nphy_offset: 0x{:x}..0x{:x}\nmemory size: 0x{:x}\n",
        kernel_img.0 as usize,
        kernel_img.1,
        phy_offset,
        limine::get_phy_offset_end(),
        *MEMORY_SIZE
    );

    let kernel_img_addr = unsafe { &*kernel_img.0 };
    let elf = utils::elf::Elf::parse(kernel_img_addr).unwrap();
    let test = utils::elf::Elf::parse(unsafe { &*TEST_ELF.as_ptr() }).unwrap();
    test.debug();

    unsafe {
        KERNEL = Some(Kernel {
            phy_offset,
            rsdp_addr: limine::rsdp_addr(),
            frame_allocator: RegionAllocator::new(),
            elf,
        });
    }

    // initing the arch
    arch::init();

    unsafe {
        memory::init(get_phy_offset_end());
        vfs::init();

        let (buffer, info) = limine::get_framebuffer();
        let terminal: Terminal<'static> = Terminal::init(buffer, info);
        TERMINAL = Some(terminal);
    }

    serial!("kernel init phase 1 done\n");

    unsafe {
        let mut scheduler = Scheduler::init(kmain as usize, "kernel");

        scheduler.create_process(terminal::shell as usize, "shell", ProcessFlags::empty());
        SCHEDULER = Some(scheduler);

        restore_cpu_status(&(*SCHEDULER.as_ref().unwrap().current_process).context)
    }
}

#[no_mangle]
fn kstart() -> ! {
    let rsp: u64;
    unsafe {
        asm!("cli");
        asm!("mov {}, rsp", out(reg) rsp);
    }
    serial!("rsp: 0x{:x}\n", rsp);

    kinit();
    serial!("failed context switching to kmain! ...\n");
    khalt()
}

#[no_mangle]
fn kmain() -> ! {
    serial!("Hello, world!, running tests...\n");

    #[cfg(feature = "test")]
    test::testing_module::test_main();

    println!("finished running tests...");
    println!(
        "\\[fg: (0, 255, 0) ||Boot success! press ctrl + shift + C to clear screen (and enter input mode)\n||]"
    );

    serial!("finished initing...\n");
    serial!("idle!\n");
    khalt()
}

// whenever a key is pressed this function should be called
// this executes a few other kernel-functions
pub fn __navi_key_pressed(key: Key) {
    if globals::terminal_inited() {
        terminal().on_key_pressed(key)
    }
}
