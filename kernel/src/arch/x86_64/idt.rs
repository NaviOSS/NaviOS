use lazy_static::lazy_static;

use crate::terminal::framebuffer::{kwrite, kwriteln};
use core::arch::asm;

pub struct IDTDescriptor {
    limit: u16,
    base: usize,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct GateDescriptor {
    offset0: u16,
    selector: u16,
    ist: u8,
    attributes: u8, // gate_type, dpl, zero and present bit
    offset1: u16,
    offset2: u32,
    reserved: u32,
}
const ATTR_TRAP: u8 = 0xF;
const ATTR_INT: u8 = 0xE;

impl GateDescriptor {
    pub fn new<T>(handler: &T, attributes: u8) -> Self {
        let offset = (handler as *const T) as u64;
        Self {
            offset0: offset as u16,
            selector: 0x08,
            ist: 0,
            attributes: attributes | 1 << 7, // attaching present attriubute
            offset1: (offset >> 16) as u16,
            offset2: (offset >> 32) as u32,
            reserved: 0,
        }
    }

    pub const fn default() -> Self {
        Self {
            offset0: 0,
            selector: 0,
            ist: 0,
            attributes: 0,
            offset1: 0,
            offset2: 0,
            reserved: 0,
        }
    }
}

type IDTT = [GateDescriptor; 256];

fn create_idt(idt: &[GateDescriptor]) -> IDTT {
    let mut table = [GateDescriptor::default(); 256];

    for (index, gate) in idt.into_iter().enumerate() {
        table[index] = *gate;
    }
    table
}

lazy_static! {
    static ref IDT: IDTT = create_idt(&[
        GateDescriptor::new(&int_handler, ATTR_INT), // divide by 0
        GateDescriptor::new(&int_handler, ATTR_INT),
        GateDescriptor::new(&int_handler, ATTR_INT),
        GateDescriptor::new(&int_handler, ATTR_INT), // breakpoint
    ]);
    static ref IDTDesc: IDTDescriptor = IDTDescriptor {
        limit: (size_of::<IDTT>() - 1) as u16,
        base: (&*IDT).as_ptr() as usize
    };
}

#[repr(C)]
struct InterruptFrame {
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

extern "x86-interrupt" fn int_handler() {
    kwriteln("hi from interrupt");
    loop {}
}

pub fn init_idt() {
    unsafe {
        asm!("lidt [{}]", in(reg) &*IDTDesc);
        asm!("sti")
    }

    kwriteln("finished initing interrupts!");
}
