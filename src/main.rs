use ovmf_prebuilt;
// code for running qemu and testing, kernel src avalible at kernel

fn main() {
    let uefi_path = env!("UEFI_PATH");
    println!("uefi: {}", uefi_path);
    let _bios_path = env!("BIOS_PATH");

    let uefi = true;

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    if uefi {
        cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
        cmd.arg("-drive")
            .arg(format!("format=raw,file={uefi_path}"))
            .arg("-display")
            .arg("sdl")
            .arg("-serial")
            .arg("stdio")
            .arg("-enable-kvm")
            .arg("-m")
            .arg("512M")
            .arg("-smp")
            .arg("2");
    }
    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}
