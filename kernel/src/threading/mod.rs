use alloc::boxed::Box;

use crate::arch::CPUStatus;

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

pub struct Scheduler {
    head: Process,
    current_process: Process,
}

impl Scheduler {
    #[inline]
    pub fn init(context: CPUStatus) -> Self {
        let process = Process {
            status: ProcessStatus::Running,
            context,
            next: None,
        };

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

    pub fn add_process(&mut self, context: CPUStatus) {
        let process = Process {
            status: ProcessStatus::Waiting,
            context,
            next: None,
        };

        let mut current = &mut self.head;
        while let Some(ref mut process) = current.next {
            current = &mut **process;
        }

        current.next = Some(Box::new(process));
    }
}
