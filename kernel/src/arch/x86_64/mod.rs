mod gdt;
mod idt;

use core::arch::asm;

use crate::terminal::framebuffer::{kwrite, kwrite_hex, kwriteln};

use self::{gdt::init_gdt, idt::init_idt};

pub extern "C" fn init() {
    kwriteln("initing gdt....");
    init_gdt();

    init_idt();
    unsafe {
        asm!("int3");
    }

    let rax: u64;
    unsafe {
        asm!(
            "
                mov rax, 0xFFFFFFFFFFFFFFFF
                mov {}, rax
            ",
            out(reg) rax
        );
    }

    kwriteln("TESTS:");

    kwrite("rax: ");
    kwrite_hex(rax);

    kwrite("according to this information are we in long mode?: ");
    if rax == 0xFFFFFFFFFFFFFFFF {
        kwriteln("yes");
    } else {
        kwriteln("no");
    }
}

#[macro_export]
macro_rules! arch_init {
    () => {
        use arch::x86_64::init;
        init()
    };
}
