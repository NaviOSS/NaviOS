use core::{alloc::Layout, arch::asm};

use alloc::boxed::Box;

use crate::{arch::CPUStatus, global_allocator, VirtAddr};

pub const STACK_SIZE: usize = 4096 * 4;
pub const STACK_LAYOUT: Layout = Layout::new::<[u8; STACK_SIZE]>();

/// returns a pointer to the end of the stack
pub fn alloc_stack() -> VirtAddr {
    unsafe {
        global_allocator()
            .lock()
            .alloc_mut(STACK_LAYOUT)
            .add(STACK_SIZE) as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProcessStatus {
    Waiting,

    Running,
    WaitingForBurying,
}

#[derive(Debug, Clone)]
pub struct Process {
    status: ProcessStatus,
    context: CPUStatus,
    next: Option<Box<Process>>,
}

impl Process {
    pub fn create(function: usize) -> Self {
        let status = ProcessStatus::Waiting;
        let mut context = CPUStatus::default();

        #[cfg(target_arch = "x86_64")]
        {
            context.rsp = alloc_stack() as u64;
            context.rip = function as u64;
            context.rflags = 0x202;

            context.ss = 0x10;
            context.cs = 0x8;
        }

        Process {
            status,
            context,
            next: None,
        }
    }
}
#[derive(Debug)]
pub struct Scheduler {
    head: Box<Process>,
    // raw pointers for peformance, we are ring0 we need the lowest stuff
    current_process: *mut Process,
}

impl Scheduler {
    #[inline]
    pub fn init(process: Process) -> Self {
        let mut process = Box::new(process);
        Self {
            current_process: &mut *process,
            head: process,
        }
    }

    /// context switches into next process, takes current context outputs new context
    pub unsafe fn switch(&mut self, context: CPUStatus) -> CPUStatus {
        unsafe { asm!("cli") }

        (*self.current_process).context = context;
        (*self.current_process).status = ProcessStatus::Waiting;

        loop {
            if (*self.current_process).next.is_some() {
                self.current_process = &mut **(*self.current_process).next.as_mut().unwrap();
            } else {
                self.current_process = &mut *self.head;
            }

            if (*self.current_process).status == ProcessStatus::Waiting {
                (*self.current_process).status = ProcessStatus::Running;
                break;
            }
        }
        return (*self.current_process).context;
    }

    pub fn add_process(&mut self, process: Process) {
        let mut current = &mut *self.head;
        while let Some(ref mut process) = current.next {
            current = &mut **process;
        }

        current.next = Some(Box::new(process));
    }
    /// wrapper around `Process::create` that also adds the result to self using
    /// `Self::add_process`
    pub fn create_process(&mut self, function: usize) {
        self.add_process(Process::create(function))
    }
}
