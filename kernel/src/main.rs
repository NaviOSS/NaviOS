#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(abi_x86_interrupt)]
#![feature(iter_advance_by)]
#![feature(const_mut_refs)]
#![feature(custom_test_frameworks)]
#![feature(proc_macro_hygiene)]
#![feature(custom_inner_attributes)]
#[cfg(feature = "test")]
mod test;

mod arch;
mod memory;
mod terminal;
mod utils;

extern crate alloc;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::terminal::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => (crate::print!("{}\n", format_args!($($arg)*)));
}

#[allow(unused_imports)]
use core::panic::PanicInfo;
#[allow(unused_imports)]
use terminal::kerr;

use terminal::framebuffer::Terminal;
#[allow(dead_code)]
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("kernel panic: {}", info.message());

    kerr("\ncannot continue execution kernel will now hang");
    loop {}
}

pub extern "C" fn kinit(boot_info: &'static mut bootloader_api::BootInfo) {
    // initing terminal
    let phy_offset = &mut boot_info.physical_memory_offset;
    let regions = &mut boot_info.memory_regions;

    let terminal = Terminal::init(boot_info.framebuffer.as_mut().unwrap());
    unsafe {
        TERMINAL = Some(terminal);
    }
    // initing the arch
    arch_init!(); // macro is defined for each arch

    unsafe {
        memory::init_memory(phy_offset, regions);
    };
}

static mut TERMINAL: Option<Terminal> = None;
#[no_mangle]
fn kmain(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    kinit(boot_info);

    #[cfg(feature = "test")]
    test::testing_module::test_main();

    println!("Hello, world!");
    loop {}
}
static CONFIG: bootloader_api::BootloaderConfig = {
    use bootloader_api::{
        config::{Mapping, Mappings},
        BootloaderConfig,
    };

    let mut config = BootloaderConfig::new_default();
    let mut mappings = Mappings::new_default();
    mappings.physical_memory = Some(Mapping::Dynamic);
    config.mappings = mappings;
    config
};
bootloader_api::entry_point!(kmain, config = { &CONFIG });
