pub mod gdt;

use crate::kernel::vga::kwrite;
use crate::kernel::vga::write_hex;
use core::arch::asm;

#[inline]
fn print_registers() {
    let (ds, es, fs, gs, ss, cs, eax): (u16, u16, u16, u16, u16, u16, u32);

    unsafe {
        asm!(
            "
        mov {0:x}, ds
        mov {1:x}, es
        mov {2:x}, fs
        mov {3:x}, gs
        mov {4:x}, ss
        mov {5:x}, cs

        "
            , out(reg) ds
            , out(reg) es
            , out(reg) fs
            , out(reg) gs
            , out(reg) ss
            , out(reg) cs
        );
    }

    kwrite(s!("ds: \0"));
    write_hex(ds.into());

    kwrite(s!("es: \0"));
    write_hex(es.into());

    kwrite(s!("fs: \0"));
    write_hex(fs.into());

    kwrite(s!("gs: \0"));
    write_hex(gs.into());

    kwrite(s!("ss: \0"));
    write_hex(ss.into());

    kwrite(s!("cs: \0"));
    write_hex(cs.into());

    unsafe {
        asm!(
            "
            mov eax, 0xFFFFFFFF
            mov {0}, eax
            ",
            out(reg) eax,
        )
    }

    kwrite(s!("not xor eax: \0"));
    write_hex(eax);
}

use self::gdt::{GDTType, GDT};

pub extern "C" fn init() {
    kwrite(s!("disabling interrupts....\n\0"));
    unsafe {
        asm!(
            "
        cli
        ",
            options(nostack)
        );
    };
    kwrite(s!("initing the gdt....\n\0"));
    gdt::init_gdt();
    // entering protected mode
    unsafe {
        asm!(
            "
        mov eax, cr0
        or al, 1
        mov cr0, eax
        "
        );
    };
    kwrite(s!("initing pm....\n\0"));
    gdt::init_pm();

    kwrite(s!("init done\n\0"));

    // arch spec stuff
    kwrite(s!("Hello from x86: \n\0"));
    print_registers();

    write_hex(gdt::GDT_DESCRIPTOR.limit as u32);
    write_hex(size_of::<gdt::GDTType>() as u32);
    write_hex(gdt::GDT_DESCRIPTOR.base);
    write_hex(&*GDT as *const GDTType as u32);
}

#[macro_export]
macro_rules! arch_init {
    () => {
        use arch::x86::init;
        init()
    };
}
