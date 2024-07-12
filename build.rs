// build.rs

use std::{path::PathBuf, process::Command};

use bootloader::BootConfig;

fn main() {
    // set by cargo, build scripts should use this directory for output files
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let kernel = PathBuf::from(std::env::var_os("CARGO_BIN_FILE_KERNEL_kernel").unwrap());

    let config: BootConfig = {
        let mut config = BootConfig::default();
        let mut farmebuffer = bootloader_boot_config::FrameBuffer::default();

        // farmebuffer.minimum_framebuffer_width = Some(512);
        // farmebuffer.minimum_framebuffer_height = Some(256);

        config.frame_buffer_logging = false;
        config.serial_logging = false;
        config.log_level = bootloader_boot_config::LevelFilter::Error;
        config.frame_buffer = farmebuffer;

        config
    };
    // create an UEFI disk image (optional)
    let uefi_path = out_dir.join("uefi.img");
    bootloader::UefiBoot::new(&kernel)
        .set_boot_config(&config)
        .create_disk_image(&uefi_path)
        .unwrap();

    // create a BIOS disk image
    let bios_path = out_dir.join("bios.img");

    bootloader::BiosBoot::new(&kernel)
        .set_boot_config(&config)
        .create_disk_image(&bios_path)
        .unwrap();

    // pass the disk image paths as env variables to the `main.rs`
    println!("cargo:rustc-env=UEFI_PATH={}", uefi_path.display());
    println!("cargo:rustc-env=BIOS_PATH={}", bios_path.display());
}
