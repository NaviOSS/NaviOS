use core::{alloc::Layout, arch::asm};

use alloc::{boxed::Box, vec::Vec};

use crate::{
    arch::threading::CPUStatus, global_allocator, memory::paging::PageTable, paging_mapper,
    phy_offset, serial, VirtAddr,
};

pub const STACK_SIZE: usize = 4096 * 4;
pub const STACK_LAYOUT: Layout = Layout::new::<[u8; STACK_SIZE]>();

/// helper function to work with `name` in Process
fn trim_trailing_zeros(slice: &[u8]) -> &[u8] {
    if let Some(last_non_zero) = slice.iter().rposition(|&x| x != 0) {
        &slice[..=last_non_zero]
    } else {
        &[]
    }
}

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
pub enum ProcessStatus {
    Waiting,

    Running,
    WaitingForBurying,
}

#[derive(Debug, Clone)]
pub struct Process {
    pub pid: u64,
    pub name: [u8; 64],
    pub status: ProcessStatus,
    pub context: CPUStatus,

    pub root_page_table: *mut PageTable,
    pub stack_end: *mut u8,
    pub next: Option<Box<Process>>,
}

impl Process {
    pub fn create(function: usize, pid: u64, name: &str) -> Self {
        let name_bytes = name.as_bytes();

        let mut name = [0u8; 64];

        let len = name_bytes.len().min(64);
        name[..len].copy_from_slice(&name_bytes[..len]);

        let status = ProcessStatus::Waiting;
        let mut context = CPUStatus::default();

        let stack_end = alloc_stack() as *mut u8;
        let root_page_table = paging_mapper().allocate_pml4().unwrap();

        #[cfg(target_arch = "x86_64")]
        {
            context.rsp = stack_end as u64;
            context.rip = function as u64;
            context.rflags = 0x202;

            context.ss = 0x10;
            context.cs = 0x8;
            context.cr3 = root_page_table as u64;
        }

        let root_page_table = (root_page_table + phy_offset()) as *mut PageTable;

        Process {
            pid,
            name,
            status,
            context,

            stack_end,
            root_page_table,
            next: None,
        }
    }

    /// frees self and then returns next
    /// frees all resources that has something to do with this process even the process stack and
    /// page table
    /// TODO: test this properly
    pub fn free(&mut self) -> Option<Box<Process>> {
        serial!("deallocating a process! ...\n");

        unsafe {
            global_allocator()
                .lock()
                .dealloc_mut(self.stack_end.sub(STACK_SIZE), STACK_LAYOUT);
        }

        serial!("deallocated the stack!\n");

        let root_page_table = unsafe { &mut (*self.root_page_table) };
        unsafe { root_page_table.free(4) };
        serial!("deallocated the root page table!\n");

        self.next.take()
    }
}
#[derive(Debug)]
pub struct Scheduler {
    pub head: Box<Process>,
    /// raw pointers for peformance, we are ring0 we need the lowest stuff
    pub current_process: *mut Process,
    next_pid: u64,
}

impl Scheduler {
    #[inline]
    pub fn init(function: usize, name: &str) -> Self {
        let mut process = Box::new(Process::create(function, 0, name));
        Self {
            current_process: &mut *process,
            head: process,
            next_pid: 1,
        }
    }

    /// context switches into next process, takes current context outputs new context
    pub unsafe fn switch(&mut self, context: CPUStatus) -> CPUStatus {
        unsafe { asm!("cli") }

        (*self.current_process).context = context;

        if (*self.current_process).status != ProcessStatus::WaitingForBurying {
            (*self.current_process).status = ProcessStatus::Waiting;
        }

        loop {
            if (*self.current_process)
                .next
                .as_ref()
                .is_some_and(|x| x.status == ProcessStatus::WaitingForBurying)
            {
                (*self.current_process).next =
                    (*self.current_process).next.as_mut().unwrap().free();
            }

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

    /// appends a process to the end of the scheduler head
    fn add_process(&mut self, process: Process) {
        let mut current = &mut *self.head;
        while let Some(ref mut process) = current.next {
            current = &mut **process;
        }

        current.next = Some(Box::new(process));
    }

    /// sets a process with pid `pid` status to WaitingForBurying returns Err(()) if there is no
    /// such a process
    pub fn pkill(&mut self, pid: u64) -> Result<(), ()> {
        let mut current = &mut *self.head;
        let mut found = false;
        while let Some(ref mut process) = current.next {
            if current.pid == pid {
                found = true;
                break;
            }

            current = &mut **process;
            if current.pid == pid {
                found = true;
                break;
            }
        }

        if !found {
            Err(())
        } else {
            current.status = ProcessStatus::WaitingForBurying;
            Ok(())
        }
    }

    /// sets all process(s) with name `name` status to WaitingForBurying returns Err(()) if there is no
    /// such a process
    /// current implentation just collects all the pids and executes `Self::pkill`
    /// TODO: work on better kill implentations for now this works
    pub fn pkillall(&mut self, name: &[u8]) -> Result<(), ()> {
        let mut current = &mut *self.head;
        let mut plist = Vec::new();

        while let Some(ref mut process) = current.next {
            if trim_trailing_zeros(&current.name) == name {
                plist.push(current.pid);
                break;
            }

            current = &mut **process;
            if trim_trailing_zeros(&current.name) == name {
                plist.push(current.pid);
                break;
            }
        }

        if plist.is_empty() {
            Err(())
        } else {
            for pid in plist {
                self.pkill(pid)?
            }

            Ok(())
        }
    }

    /// wrapper around `Process::create` that also adds the result to self using
    /// `Self::add_process`
    pub fn create_process(&mut self, function: usize, name: &str) {
        self.add_process(Process::create(function, self.next_pid, name));
        self.next_pid += 1;
    }
}
