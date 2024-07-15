mod gdt;
mod interrupts;

use interrupts::{enable_interrupts, init_idt};

use self::gdt::init_gdt;

pub extern "C" fn init() {
    init_gdt();
    init_idt();
    enable_interrupts();
}

#[macro_export]
macro_rules! arch_init {
    () => {
        use arch::x86_64::init;
        init()
    };
}
