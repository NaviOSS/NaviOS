mod gdt;
mod idt;

use core::arch::asm;

use crate::{print, println};

use self::{gdt::init_gdt, idt::init_idt};

pub extern "C" fn init() {
    println!("initing gdt....");
    init_gdt();

    init_idt();
    unsafe {
        asm!("int3");
    }

    // fn stack_overflow() {
    //     stack_overflow(); // for each recursion, the return address is pushed
    // }

    // // trigger a stack overflow
    // stack_overflow();

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

    println!("TESTS:");

    println!("rax: {:#018x}", rax);

    print!("according to this information are we in long mode?: ");
    if rax == 0xFFFFFFFFFFFFFFFF {
        println!("yes");
    } else {
        println!("no");
    }
}

#[macro_export]
macro_rules! arch_init {
    () => {
        use arch::x86_64::init;
        init()
    };
}
