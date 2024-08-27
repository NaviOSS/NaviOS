fn main() {
    println!("cargo:rustc-link-arg=-Tkernel/linker.ld");
    println!("cargo:rerun-if-changed=kernel/linker.ld");
}
