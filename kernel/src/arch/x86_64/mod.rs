mod gdt;

use core::arch::asm;

use crate::terminal::framebuffer::{kwrite, kwrite_hex, kwriteln};

use self::gdt::init_gdt;

pub extern "C" fn init() {
    kwriteln("initing gdt....");
    init_gdt();

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
        kwriteln("none");
    }
}

#[macro_export]
macro_rules! arch_init {
    () => {
        use arch::x86_64::init;
        init()
    };
}
