// build.rs

use std::{env::current_dir, fs, path::PathBuf, process::Command};

/// TODO: spilt into more functions and make it work on other oses like windows
fn main() {
    // set by cargo, build scripts should use this directory for output files
    let kernel = PathBuf::from(std::env::var_os("CARGO_BIN_FILE_KERNEL_kernel").unwrap());

    if !fs::exists("limine").unwrap() {
        Command::new("git")
            .arg("clone")
            .arg("https://github.com/limine-bootloader/limine.git")
            .arg("--branch=v8.x-binary")
            .arg("--depth=1")
            .output()
            .unwrap();
    }

    Command::new("make")
        .arg("-C")
        .arg("limine")
        .output()
        .unwrap();

    fs::create_dir_all("iso_root/boot/limine").unwrap();

    Command::new("mv")
        .arg("-v")
        .arg(kernel)
        .arg("iso_root/boot/kernel")
        .output()
        .unwrap();

    Command::new("cp")
        .arg("-v")
        .arg("limine.conf")
        .arg("limine/limine-bios.sys")
        .arg("limine/limine-bios-cd.bin")
        .arg("limine/limine-uefi-cd.bin")
        .arg("iso_root/boot/limine")
        .output()
        .unwrap();

    fs::create_dir_all("iso_root/EFI/BOOT").unwrap();
    Command::new("cp")
        .arg("-v")
        .arg("limine/BOOTX64.EFI")
        .arg("iso_root/EFI/BOOT")
        .output()
        .unwrap();

    Command::new("cp")
        .arg("-v")
        .arg("limine/BOOTIA32.EFI")
        .arg("iso_root/EFI/BOOT")
        .output()
        .unwrap();

    // command too long ):
    Command::new("bash")
        .arg("-c")
        .arg(
            "xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table \
		--efi-boot boot/limine/limine-uefi-cd.bin \
		-efi-boot-part --efi-boot-image --protective-msdos-label \
		iso_root -o navios.iso
",
        )
        .output()
        .unwrap();

    fs::remove_dir_all("iso_root").unwrap();

    let iso_path = current_dir().unwrap().join("navios.iso");
    println!("cargo:rerun-if-changed={}", iso_path.display());
    println!("cargo:rerun-if-changed={}", "limine");

    // pass the disk image paths as env variables to the `main.rs`
    println!("cargo:rustc-env=ISO_PATH={}", iso_path.display());
}
