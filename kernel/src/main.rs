#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
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
use alloc::boxed::Box;
use arch::x86_64::serial;

use drivers::keyboard::Key;
use drivers::vfs;
use drivers::vfs::VFS;
use globals::*;

use limine::get_phy_offset;
use limine::get_phy_offset_end;
use limine::MEMORY_SIZE;
use memory::frame_allocator::RegionAllocator;
pub use memory::PhysAddr;
pub use memory::VirtAddr;
use spin::Mutex;
use terminal::framebuffer::Terminal;
use threading::processes::ProcessFlags;
use threading::Scheduler;

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
        #[cfg(target_arch = "x86_64")]
        unsafe {
            asm!("hlt")
        }
    }
}

#[allow(unused_imports)]
use core::panic::PanicInfo;
use core::slice;

/// prints to both the serial and the terminal doesn't print to the terminal if it panicked or if
/// it is not ready...
#[macro_export]
macro_rules! cross_println {
    ($($arg:tt)*) => {
        crate::serial!($($arg)*);
        crate::serial!("\n");

        if !crate::terminal().panicked {
            crate::terminal().panicked = true;

            crate::println!(r"{}", format_args!($($arg)*));
            crate::terminal().panicked = false;
        }
    };
}

/// runtime debug info that is only avalible though test feature
/// takes a $mod and an Arguments, mod must be a type
#[macro_export]
macro_rules! debug {
    ($mod: path, $($arg:tt)*) => {
        // makes sure $mod is a vaild type
        let _ = core::marker::PhantomData::<$mod>;
        crate::serial!("\x1B[38;2;0;155;200m[DEBUG]\x1B[38;2;255;155;0m {}: \x1B[0m{}\n", stringify!($mod), format_args!($($arg)*));
    };
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe { asm!("cli") }
    unsafe {
        arch::x86_64::serial::SERIAL.inner.force_unlock();
    }
    terminal().enter_panic();

    cross_println!(
        "kernel panic:\n{}, at {}",
        info.message(),
        info.location().unwrap()
    );
    print_stack_trace();

    crate::serial!("tty stdout dump:\n{}\n", crate::terminal().stdout_buffer);
    crate::serial!("tty stdout dump:\n{}\n", crate::terminal().stdin_buffer);
    khalt()
}

#[allow(unused)]
fn print_stack_trace() {
    let mut fp: usize;

    unsafe {
        core::arch::asm!("mov {}, rbp", out(reg) fp);

        cross_println!("stack trace: ");
        while fp != 0 {
            let return_address_ptr = (fp as *const usize).offset(1);
            let return_address = *return_address_ptr;

            let name = {
                if kernel_inited() {
                    let sym = kernel().elf.sym_from_value_range(return_address);

                    if let Some(sym) = sym {
                        kernel().elf.string_table_index(sym.name_index)
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
    arch::init_phase1();
    Terminal::init();
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

    let kernel_img_bytes = unsafe { slice::from_raw_parts(kernel_img.0, kernel_img.1) };
    let elf = utils::elf::Elf::new(kernel_img_bytes).unwrap();

    kernel().phy_offset = phy_offset;
    kernel().rsdp_addr = limine::rsdp_addr();
    kernel().frame_allocator = Mutex::new(RegionAllocator::new());
    kernel().elf = elf;

    memory::init(get_phy_offset_end());
    Terminal::init_finish();
    println!("Terminal initialized successfuly");

    // initing the arch
    arch::init_phase2();

    unsafe {
        vfs::init();

        let mut ramdisk = limine::get_ramdisk();
        let mut ramfs = Box::new(vfs::ramfs::RamFS::new());

        VFS::unpack_tar(&mut *ramfs, &mut ramdisk).expect("failed unpacking archive");
        vfs::vfs().mount(b"sys", ramfs).expect("failed mounting");

        debug!(Kernel, "init phase 1 done");

        Scheduler::init(kmain as usize, "kernel");
    }
}

#[no_mangle]
fn kstart() -> ! {
    kinit();
    serial!("failed context switching to kmain! ...\n");
    khalt()
}

#[no_mangle]
fn kmain() -> ! {
    debug!(Scheduler, "done ...");
    scheduler().create_process(
        terminal::shell as usize,
        "shell",
        &[],
        ProcessFlags::empty(),
    );

    serial!("Hello, world!, running tests...\n");

    #[cfg(feature = "test")]
    test::testing_module::test_main();

    println!("finished running tests...");
    println!(
        "\x1B[38;2;0;255;0mBoot success! press ctrl + shift + C to clear screen (and enter input mode)\x1B[0m"
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
