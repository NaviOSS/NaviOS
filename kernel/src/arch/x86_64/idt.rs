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
    pub ist: u8,
    attributes: u8, // gate_type, dpl, zero and present bit
    offset1: u16,
    offset2: u32,
    reserved: u32,
}
const ATTR_TRAP: u8 = 0xF;
const ATTR_INT: u8 = 0xE;

impl GateDescriptor {
    pub const fn new(handler: u64, attributes: u8) -> Self {
        let offset = handler;
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
macro_rules! create_idt {
    ($(($indx: literal, $handler: tt, $attributes: expr $(, $ist: literal)?)),*) => {
        {
            let mut table = EMPTY_TABLE;

            #[allow(unused_mut)]
            #[allow(unused_assignments)]
            for (index, handler, attributes, ist) in &[$(($indx, $handler as u64, $attributes,  {let mut ist = None; $(ist = Some($ist as i8);)? ist.unwrap_or(-1) + 1}), )*] {
                table[*index as usize] = GateDescriptor::new(*handler, *attributes);
                table[*index as usize].ist = *ist as u8;
            }
            table
        }
    };
}

lazy_static! {
    static ref IDT: IDTT = create_idt!(
        (0, divide_by_zero_handler, ATTR_INT),
        (3, breakpoint_handler, ATTR_INT),
        (8, dobule_fault_handler, ATTR_TRAP, 0),
        (14, page_fault_handler, ATTR_TRAP)
    );
    static ref IDTDesc: IDTDescriptor = IDTDescriptor {
        limit: (size_of::<IDTT>() - 1) as u16,
        base: (&*IDT).as_ptr() as usize
    };
}

#[derive(Debug)]
#[repr(C, packed)]
struct InterruptFrame {
    insturaction: u64,
    code_segment: u64,
    flags: u64,
    stack_pointer: u64,
    stack_segment: u64,
}

#[derive(Debug)]
#[repr(C, packed)]
struct TrapFrame {
    insturaction: u64,
    code_segment: u64,
    flags: u64,
    stack_pointer: u64,
    stack_segment: u64,
    error_code: u64,
}

extern "x86-interrupt" fn divide_by_zero_handler(frame: InterruptFrame) {
    panic!("divide by zero exception\nframe: {:#?}", frame);
}

extern "x86-interrupt" fn breakpoint_handler(frame: InterruptFrame) {
    println!("hi from interrupt, breakpoint!, {:#?}", frame);
}

extern "x86-interrupt" fn dobule_fault_handler(frame: TrapFrame) {
    panic!("double fault exception\nframe: {:#?}", frame);
}

extern "x86-interrupt" fn page_fault_handler(frame: TrapFrame) {
    panic!("page fault exception\nframe: {:#?}", frame)
}

pub fn init_idt() {
    unsafe {
        asm!("lidt [{}]", in(reg) &*IDTDesc, options(nostack));
        asm!("sti")
    }

    println!("finished initing interrupts!");
}
