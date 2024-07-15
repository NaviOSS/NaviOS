mod handlers;
use lazy_static::lazy_static;

use x86_64::{instructions::interrupts, structures::idt::InterruptDescriptorTable};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = unsafe {
        let mut idt = InterruptDescriptorTable::new();
        idt.divide_error
            .set_handler_fn(handlers::divide_by_zero_handler);
        idt.breakpoint.set_handler_fn(handlers::breakpoint_handler);
        idt.page_fault.set_handler_fn(handlers::page_fault_handler);
        idt.double_fault
            .set_handler_fn(handlers::double_fault_handler)
            .set_stack_index(0);
        idt
    };
}
pub fn init_idt() {
    IDT.load();
}

pub fn enable_interrupts() {
    interrupts::enable();
}
