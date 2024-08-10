use core::alloc::Layout;

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

pub struct Scheduler {
    head: Process,
    current_process: Process,
}

impl Scheduler {
    #[inline]
    pub fn init(process: Process) -> Self {
        Self {
            head: process.clone(),
            current_process: process,
        }
    }

    /// context switches into next process, takes current context outputs new context
    pub fn switch(&mut self, context: CPUStatus) -> CPUStatus {
        self.current_process.context = context;
        self.current_process.status = ProcessStatus::Waiting;

        loop {
            if self.current_process.next.is_some() {
                self.current_process = *self.current_process.next.take().unwrap();
            } else {
                self.current_process = self.head.clone();
            }

            if !(self.current_process.status == ProcessStatus::WaitingForBurying) {
                self.current_process.status = ProcessStatus::Running;
                break;
            }
        }

        return self.current_process.context;
    }

    pub fn add_process(&mut self, process: Process) {
        let mut current = &mut self.head;
        while let Some(ref mut process) = current.next {
            current = &mut **process;
        }

        current.next = Some(Box::new(process));
    }
}
