use std::env::args;

use ovmf_prebuilt;
// code for running qemu and testing, kernel src avalible at kernel

fn main() {
    let mut args = args();
    args.next();

    let iso_path = env!("ISO_PATH");

    let uefi = true;

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    if uefi {
        cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
        cmd.arg("-drive")
            .arg(format!("format=raw,file={iso_path}"))
            .arg("-display")
            .arg("sdl")
            .arg("-serial")
            .arg("stdio")
            .arg("-m")
            .arg("512M")
            .arg("-smp")
            .arg("2");
    }

    if args.next() != Some("no-kvm".to_string()) {
        cmd.arg("-enable-kvm");
    }

    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}
