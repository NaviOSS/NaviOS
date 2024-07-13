use lazy_static::lazy_static;

use core::arch::asm;

use crate::println;

type IDTT = [GateDescriptor; 256];
type HandlerFn<T> = extern "x86-interrupt" fn(T);

#[repr(C, packed)]
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
    pub fn new<T>(handler: HandlerFn<T>, attributes: u8) -> Self {
        let offset = handler as u64;
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

const EMPTY_TABLE: IDTT = [GateDescriptor::default(); 256]; // making sure it is made at compile-time

// interrupt index(code), handler, attributes
fn create_idt<T>(idt: &[(u8, HandlerFn<T>, u8)]) -> IDTT {
    let mut table = EMPTY_TABLE;

    for (index, handler, attributes) in idt {
        table[*index as usize] = GateDescriptor::new(*handler, *attributes);
    }
    table
}

lazy_static! {
    static ref IDT: IDTT = create_idt(&[(3, breakpoint_handler, ATTR_INT)]);
    static ref IDTDesc: IDTDescriptor = IDTDescriptor {
        limit: (size_of::<IDTT>() - 1) as u16,
        base: (&*IDT).as_ptr() as usize
    };
}
#[derive(Debug)]
#[repr(C, packed)]
struct InterruptFrame {
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

extern "x86-interrupt" fn breakpoint_handler(frame: InterruptFrame) {
    println!("hi from interrupt, breakpoint!, {:#?}", frame);
}

pub fn init_idt() {
    unsafe {
        asm!("lidt [{}]", in(reg) &*IDTDesc, options(nostack));
        asm!("sti")
    }

    println!("finished initing interrupts!");
}
