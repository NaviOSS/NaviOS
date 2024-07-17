use lazy_static::lazy_static;

use super::idt::{GateDescriptor, IDTT};
use super::idt::{InterruptFrame, TrapFrame};

use crate::println;
const ATTR_TRAP: u8 = 0xF;
const ATTR_INT: u8 = 0xE;
const EMPTY_TABLE: IDTT = [GateDescriptor::default(); 256]; // making sure it is made at compile-time

macro_rules! create_idt {
    ($(($indx:literal, $handler:expr, $attributes:expr $(, $ist:literal)?)),*) => {
        {
            let mut table = EMPTY_TABLE;
            $(
                let index: usize = $indx as usize;
                let handler: u64 = $handler as u64;
                let attributes: u8 = $attributes;
                let ist: u8 = {
                    #[allow(unused_variables)]
                    let ist_value = -1;
                    $(let ist_value = $ist as i8;)?
                    (ist_value + 1) as u8
                };
                table[index] = GateDescriptor::new(handler, attributes);
                table[index].ist = ist;
            )*
            table
        }
    };
}

lazy_static! {
    pub static ref IDT: IDTT = create_idt!(
        (0, divide_by_zero_handler, ATTR_INT),
        (3, breakpoint_handler, ATTR_INT),
        (8, dobule_fault_handler, ATTR_TRAP, 0),
        (14, page_fault_handler, ATTR_TRAP),
        (0x20, timer_interrupt_handler, ATTR_INT)
    );
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

extern "x86-interrupt" fn timer_interrupt_handler(frame: InterruptFrame) {
    println!("got timer {:#?}", frame);
}
