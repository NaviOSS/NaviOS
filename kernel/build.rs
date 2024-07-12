// use std::process::Command;
// fn main() {
//     #[cfg(target_arch = "x86_64")]
//     {
//         Command::new("objcopy")
//             .arg("-O")
//             .arg("elf64-x86-64")
//             .arg("-B")
//             .arg("i386")
//             .arg("-I")
//             .arg("binary")
//             .arg("../fonts/ter.psf")
//             .arg("../fonts/font.o")
//             .spawn()
//             .unwrap()
//             .wait()
//             .unwrap();

//         Command::new("ar")
//             .arg("rcs")
//             .arg("../fonts/libfont.a")
//             .arg("../fonts/font.o")
//             .spawn()
//             .unwrap()
//             .wait()
//             .unwrap();

//         println!("cargo:rerun-if-changed={}", "./fonts/libfont.a");
//         println!("cargo:rustc-link-search={}", "./fonts");
//         println!("cargo:rustc-link-lib=static={}", "font");
//     }
// }
fn main() {}
