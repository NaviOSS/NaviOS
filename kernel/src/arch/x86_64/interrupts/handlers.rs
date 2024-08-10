use core::arch::asm;

use lazy_static::lazy_static;

use super::idt::{GateDescriptor, IDTT};
use super::idt::{InterruptFrame, TrapFrame};

use crate::arch::x86_64::inb;
use crate::arch::x86_64::interrupts::apic::send_eoi;
use crate::arch::x86_64::threading::restore_cpu_status;
use crate::arch::CPUStatus;
use crate::{drivers, println, scheduler, scheduler_inited, serial};
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
        (13, general_protection_fault_handler, ATTR_TRAP),
        (14, page_fault_handler, ATTR_TRAP),
        (0x20, timer_interrupt_handler, ATTR_INT),
        (0x21, keyboard_interrupt_handler, ATTR_INT)
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

extern "x86-interrupt" fn general_protection_fault_handler(frame: TrapFrame) {
    panic!("general protection fault\nframe: {:#?}", frame);
}

extern "x86-interrupt" fn page_fault_handler(frame: TrapFrame) {
    panic!("page fault exception\nframe: {:#?}", frame)
}

extern "x86-interrupt" fn timer_interrupt_handler(frame: InterruptFrame) {
    serial!("tick\n");

    send_eoi();
    if scheduler_inited() {
        let mut capture = CPUStatus::save();
        capture.rsp = frame.stack_pointer;
        capture.rip = frame.insturaction;

        capture.cs = frame.code_segment;
        capture.ss = frame.stack_segment;
        capture.rflags = frame.flags;

        send_eoi();
        let switch = scheduler().switch(capture);
        unsafe {
            restore_cpu_status(&switch);
        }
    } else {
        send_eoi();
    }
}

#[inline]
pub fn handle_ps2_keyboard() {
    let key = inb(0x60);
    drivers::keyboard::encode_ps2_set_1(key);
}

pub extern "x86-interrupt" fn keyboard_interrupt_handler() {
    handle_ps2_keyboard();
    send_eoi();
}
