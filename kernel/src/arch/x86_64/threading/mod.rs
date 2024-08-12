use core::arch::global_asm;

use crate::{scheduler, scheduler_inited};

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct CPUStatus {
    pub rsp: u64,
    pub rflags: u64,
    pub ss: u64,
    pub cs: u64,

    pub rip: u64,

    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    r11: u64,
    r10: u64,
    r9: u64,
    r8: u64,

    rbp: u64,
    rdi: u64,
    rsi: u64,

    rdx: u64,
    rcx: u64,
    rbx: u64,
    rax: u64,
}

global_asm!(
    "
.global restore_cpu_status
.global context_switch_stub

restore_cpu_status:
    // push the iretq frame
    push [rdi + 16]     // push ss
    push [rdi]          // push rsp
    push [rdi + 8]      // push rflags
    push [rdi + 24]     // push cs
    push [rdi + 32]     // push rip

    
    mov r15, [rdi + 40]    
    mov r14, [rdi + 48]    
    mov r13, [rdi + 56]    
    mov r12, [rdi + 64]    
    mov r11, [rdi + 72]    
    mov r10, [rdi + 80]    
    mov r9, [rdi + 88]    
    mov r8, [rdi + 96]    

    mov rbp, [rdi + 104]
    mov rsi, [rdi + 120]

    mov rdx, [rdi + 128]
    mov rcx, [rdi + 136]
    mov rbx, [rdi + 144]
    mov rax, [rdi + 152]

    mov rdi, [rdi + 112]

    iretq

context_switch_stub:
    push rax
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

    push 0    // rip
    push 0x8  // cs
    push 0x10 // ss
    pushfq 
    push 0 // rsp
    call context_switch
    // UNREACHABLE!!!
    ud2
"
);

extern "C" {
    pub fn restore_cpu_status(status: &CPUStatus);
}

extern "x86-interrupt" {
    pub fn context_switch_stub();
}

#[no_mangle]
pub extern "C" fn context_switch(mut capture: CPUStatus, frame: super::interrupts::InterruptFrame) {
    capture.rsp = frame.stack_pointer;
    capture.rip = frame.insturaction;

    capture.cs = frame.code_segment;
    capture.ss = frame.stack_segment;
    capture.rflags = frame.flags;

    if scheduler_inited() {
        // actual context switching:
        unsafe {
            capture = scheduler().switch(capture);
        }
    }

    super::interrupts::apic::send_eoi();
    unsafe {
        restore_cpu_status(&capture);
    }
}
