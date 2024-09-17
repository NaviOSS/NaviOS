// build.rs

use std::{
    env::current_dir,
    fs::{self, File},
    path::PathBuf,
    process::{Command, Output},
};

use tar::Builder;

fn limine_make() -> Output {
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
        .unwrap()
}

fn out(mut output: Output) {
    output.stdout.append(&mut output.stderr);
    eprintln!("{}", String::from_utf8_lossy(&output.stdout))
}
fn setup_iso_root() {
    fs::create_dir_all("iso_root/boot/limine").unwrap();
    fs::create_dir_all("iso_root/EFI/BOOT").unwrap();
}

fn put_kernel_img() {
    let kernel = PathBuf::from(std::env::var_os("CARGO_BIN_FILE_KERNEL_kernel").unwrap());
    out(Command::new("mv")
        .arg("-v")
        .arg(kernel)
        .arg("iso_root/boot/kernel")
        .output()
        .unwrap());
}

fn put_limine_config() {
    out(Command::new("cp")
        .arg("-v")
        .arg("limine.conf")
        .arg("limine/limine-bios.sys")
        .arg("limine/limine-bios-cd.bin")
        .arg("limine/limine-uefi-cd.bin")
        .arg("iso_root/boot/limine")
        .output()
        .unwrap())
}

fn put_boot_files() {
    out(Command::new("cp")
        .arg("-v")
        .arg("limine/BOOTX64.EFI")
        .arg("iso_root/EFI/BOOT")
        .output()
        .unwrap());

    out(Command::new("cp")
        .arg("-v")
        .arg("limine/BOOTIA32.EFI")
        .arg("iso_root/EFI/BOOT")
        .output()
        .unwrap());
}

fn make_iso() {
    // command too long ):
    out(Command::new("bash")
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
        .unwrap())
}

fn compile_programs() -> Output {
    Command::new("make")
        .arg("-C")
        .arg("programs")
        .output()
        .unwrap()
}

fn make_ramdisk() {
    let file = File::create("iso_root/boot/ramdisk.tar").unwrap();
    let mut tar_builder = Builder::new(file);
    tar_builder
        .append_dir_all("programs", "programs/build")
        .unwrap();
    tar_builder.finish().unwrap();
}

fn cleanup() {
    fs::remove_dir_all("iso_root").unwrap();
}
/// TODO: spilt into more functions and make it work on other oses like windows
fn main() {
    out(limine_make());
    setup_iso_root();

    put_kernel_img();
    put_limine_config();
    put_boot_files();

    out(compile_programs());
    make_ramdisk();
    make_iso();

    let iso_path = current_dir().unwrap().join("navios.iso");
    println!("cargo:rerun-if-changed={}", iso_path.display());
    println!("cargo:rerun-if-changed={}", "limine");
    println!("cargo:rerun-if-changed={}", "programs/build");
    println!("cargo:rerun-if-changed={}", "programs");

    // pass the disk image paths as env variables to the `main.rs`
    println!("cargo:rustc-env=ISO_PATH={}", iso_path.display());
}
