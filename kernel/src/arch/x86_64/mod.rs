use core::arch::asm;

pub extern "C" fn init() {
    // panic!("OH NO THEY ARE USING X86_64!\0");
}

#[macro_export]
macro_rules! arch_init {
    () => {
        use arch::x86_64::init;
        init()
    };
}

#[macro_export]
macro_rules! header {
    () => {
        multiboot2_header!(4); // we will bootstarp this to 64bit mode
    };
}
