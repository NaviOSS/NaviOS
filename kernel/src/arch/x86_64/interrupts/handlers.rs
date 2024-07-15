use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};

use crate::println;

pub extern "x86-interrupt" fn divide_by_zero_handler(frame: InterruptStackFrame) {
    panic!("divide by zero exception\nframe: {:#?}", frame);
}

pub extern "x86-interrupt" fn breakpoint_handler(frame: InterruptStackFrame) {
    println!("hi from interrupt, breakpoint!, {:#?}", frame);
}

pub extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, _code: u64) -> ! {
    panic!("double fault exception\nframe: {:#?}", frame)
}

pub extern "x86-interrupt" fn page_fault_handler(
    frame: InterruptStackFrame,
    code: PageFaultErrorCode,
) {
    panic!(
        "page fault exception\nerror code: {:?}\naccessed address: {:?}\nframe: {:#?}",
        x86_64::registers::control::Cr2::read(),
        code,
        frame
    )
}

pub extern "x86-interrupt" fn timer_interrupt_handler(frame: InterruptStackFrame) {
    println!("got timer {:#?}", frame);
}
