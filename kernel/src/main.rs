#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(iter_advance_by)]
#![feature(const_mut_refs)]
#![feature(custom_test_frameworks)]
#![feature(proc_macro_hygiene)]
#![feature(asm_const)]
#[cfg(feature = "test")]
mod test;

mod arch;
mod drivers;
mod globals;
mod memory;
mod terminal;
mod threading;
mod utils;

extern crate alloc;
use arch::threading::restore_cpu_status;
use arch::x86_64::serial;
use bootloader_api::info::MemoryRegions;

use drivers::keyboard::Key;
use drivers::vfs::vfs_init;
use globals::*;

use memory::frame_allocator::RegionAllocator;
pub use memory::PhysAddr;
pub use memory::VirtAddr;
use terminal::framebuffer::Terminal;
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
#[inline]
pub fn khalt() -> ! {
    loop {
        unsafe { asm!("hlt") }
    }
}

#[allow(unused_imports)]
use core::panic::PanicInfo;
static mut _PANICED_AT_TERMINAL: bool = false;

/// prints to both the serial and the terminal doesn't print to the terminal if it panicked or if
/// it is not ready...
#[allow(unused)]
macro_rules! cross_println {
    ($($arg:tt)*) => {
        serial!($($arg)*);
        serial!("\n");
        if terminal_inited() && !unsafe { _PANICED_AT_TERMINAL } {
            println!($($arg)*);
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
            cross_println!("  {:#x}", return_address);
            fp = *(fp as *const usize);
        }
    }
}

pub extern "C" fn kinit(bootinfo: &'static mut bootloader_api::BootInfo) {
    // initing globals
    let phy_offset = &mut bootinfo.physical_memory_offset;
    let phy_offset = phy_offset.as_mut().unwrap();

    let regions: &'static mut MemoryRegions = &mut bootinfo.memory_regions;
    let phy_offset = *phy_offset as usize;

    serial!(
        "image: 0x{:x}\nlen: 0x{:x}\nphy_offset: 0x{:x}\n",
        bootinfo.kernel_image_offset,
        bootinfo.kernel_len,
        phy_offset
    );

    unsafe {
        KERNEL = Some(Kernel {
            phy_offset,
            rsdp_addr: bootinfo.rsdp_addr.into(),
            frame_allocator: RegionAllocator::new(&mut *regions, phy_offset),
        });
    }

    // initing the arch
    arch::init();
    unsafe {
        memory::init((bootinfo.kernel_image_offset + bootinfo.kernel_len + 1) as usize);
        vfs_init();

        let terminal: Terminal<'static> = Terminal::init(bootinfo.framebuffer.as_mut().unwrap());
        TERMINAL = Some(terminal);
    }

    serial!("kernel init phase 1 done\n");

    unsafe {
        let mut scheduler = Scheduler::init(kmain as usize, "kernel");

        scheduler.create_process(terminal::shell as usize, "shell");
        SCHEDULER = Some(scheduler);

        restore_cpu_status(&(*SCHEDULER.as_ref().unwrap().current_process).context)
    }
}

#[no_mangle]
fn kstart(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    let rsp: u64;
    unsafe {
        asm!("cli");
        asm!("mov {}, rsp", out(reg) rsp);
    }
    serial!("rsp: 0x{:x}\n", rsp);

    kinit(boot_info);
    serial!("failed context switching to kmain! ...\n");
    khalt()
}

fn kmain() -> ! {
    serial!("Hello, world!, running tests...\n");

    #[cfg(feature = "test")]
    test::testing_module::test_main();

    println!(
        "\\[fg: (0, 255, 0) ||Boot success! press ctrl + shift + C to clear screen (and enter input mode)\n||]"
    );

    serial!("finished initing...\n");
    serial!("idle!\n");

    khalt()
}

// /// does some pooling and stuff stops interrupts to do it's work first!
// fn kwork() {
//     serial!("work!\n");
//     loop {
//         // unsafe { asm!("cli") }
//         // #[cfg(target_arch = "x86_64")]
//         // arch::x86_64::interrupts::handlers::handle_ps2_keyboard();
//         // unsafe { asm!("sti") }
//     }
// }

// whenever a key is pressed this function should be called
// this executes a few other kernel-functions
pub fn __navi_key_pressed(key: Key) {
    if globals::terminal_inited() {
        terminal().on_key_pressed(key)
    }
}

static CONFIG: bootloader_api::BootloaderConfig = {
    use bootloader_api::{
        config::{Mapping, Mappings},
        BootloaderConfig,
    };

    let mut config = BootloaderConfig::new_default();
    let mut mappings = Mappings::new_default();
    mappings.physical_memory = Some(Mapping::Dynamic);
    mappings.dynamic_range_start = Some(0xffff_8000_0000_0000);
    config.mappings = mappings;
    config
};
bootloader_api::entry_point!(kstart, config = { &CONFIG });
