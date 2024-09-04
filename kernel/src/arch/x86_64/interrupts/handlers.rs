use core::arch::{asm, global_asm};

use lazy_static::lazy_static;

use super::idt::{GateDescriptor, IDTT};
use super::{InterruptFrame, TrapFrame};

use crate::arch::x86_64::interrupts::apic::send_eoi;
use crate::arch::x86_64::{inb, threading};
use crate::threading::ProcessStatus;
use crate::{drivers, println, scheduler};

const ATTR_TRAP: u8 = 0xF;
const ATTR_INT: u8 = 0xE;
const ATTR_RING3: u8 = 3 << 5;

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
        (6, invaild_opcode, ATTR_INT),
        (8, dobule_fault_handler, ATTR_TRAP, 0),
        (0xC, stack_segment_fault_handler, ATTR_TRAP, 0),
        (13, general_protection_fault_handler, ATTR_TRAP),
        (14, page_fault_handler, ATTR_TRAP),
        (0x20, threading::context_switch_stub, ATTR_INT, 1),
        (0x21, keyboard_interrupt_handler, ATTR_INT),
        (0x80, syscall_base, ATTR_INT | ATTR_RING3)
    );
}

#[no_mangle]
extern "x86-interrupt" fn divide_by_zero_handler(frame: InterruptFrame) {
    panic!("---- Divide By Zero Exception ----\n{}", frame);
}

extern "x86-interrupt" fn invaild_opcode(frame: InterruptFrame) {
    panic!("---- Invaild OPCODE ----\n{}", frame);
}

#[no_mangle]
extern "x86-interrupt" fn breakpoint_handler(frame: InterruptFrame) {
    println!("hi from interrupt, breakpoint!\n{}", frame);
}

#[no_mangle]
extern "x86-interrupt" fn dobule_fault_handler(frame: TrapFrame) {
    panic!("---- Double Fault ----\n{}", frame);
}

#[no_mangle]
extern "x86-interrupt" fn stack_segment_fault_handler(frame: TrapFrame) {
    panic!("---- Stack-Segment Fault ----\n{}", frame);
}

#[no_mangle]
extern "x86-interrupt" fn general_protection_fault_handler(frame: TrapFrame) {
    panic!("---- General Protection Fault ----\n{}", frame,);
}

#[no_mangle]
extern "x86-interrupt" fn page_fault_handler(frame: TrapFrame) {
    panic!("---- Page Fault ----\n{}", frame)
}

#[inline]
pub fn handle_ps2_keyboard() {
    let key = inb(0x60);
    drivers::keyboard::encode_ps2_set_1(key);
}
#[no_mangle]
pub extern "x86-interrupt" fn keyboard_interrupt_handler() {
    handle_ps2_keyboard();
    send_eoi();
}

global_asm!(
    "
.section .rodata
syscall_table:
    .quad sysexit
    .quad sysprint
syscall_table_end:

SYSCALL_TABLE_INFO:
    .word (syscall_table_end - syscall_table) / 8

.set KERNEL_UNSUPPORTED, 7
.section .text
.global syscall_base

syscall_base:
    cmp rax, [SYSCALL_TABLE_INFO]
    jge unsupported
    push rbx
    push rcx
    push rdx
    push rsi
    push rdi
    push rbp
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15
    call [syscall_table + rax * 8]
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rbp
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rbx
    iretq
unsupported:
    mov rax, -KERNEL_UNSUPPORTED
    iretq
"
);

extern "x86-interrupt" {
    fn syscall_base();
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// registers pushed by syscall_base
struct SyscallRegisters {
    _r15: u64,
    _r14: u64,
    _r13: u64,
    _r12: u64,
    _r11: u64,
    _r10: u64,
    _r9: u64,
    _r8: u64,
    _rbp: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
}

macro_rules! sysret {
    ($val: expr) => {
        unsafe {
            asm!("mov rax, {:r}", in(reg) $val, options(nostack));
            return;
        }
    };
}

#[no_mangle]
extern "C" fn sysprint(registers: SyscallRegisters) {
    println!("sysprint!\n {:#?}", registers);
    sysret!(0)
}

/// for now
#[no_mangle]
unsafe extern "C" fn sysexit() {
    (*scheduler().current_process).status = ProcessStatus::WaitingForBurying;

    // we cannot return if we do will will return into bad address we should wait until the
    // scheduler switches processes
    unsafe {
        asm!("sti");
        loop {
            asm!("hlt")
        }
    }
}
