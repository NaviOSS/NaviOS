pub mod expose;
pub mod processes;
pub const STACK_SIZE: usize = PAGE_SIZE * 6;
pub const STACK_START: usize = 0x00007A3000000000;
pub const STACK_END: usize = STACK_START + STACK_SIZE;

pub const RING0_STACK_START: usize = 0x00007A0000000000;
pub const RING0_STACK_END: usize = RING0_STACK_START + STACK_SIZE;

pub const ENVIROMENT_START: usize = 0x00007E0000000000;
pub const ARGV_START: usize = ENVIROMENT_START + 0xA000000000;
pub const ARGV_SIZE: usize = PAGE_SIZE * 4;

use core::{arch::asm, mem::MaybeUninit};
use processes::{Process, ProcessFlags, ProcessStatus, Resource};

use alloc::{boxed::Box, vec::Vec};

use crate::{
    arch::threading::{restore_cpu_status, CPUStatus},
    debug,
    memory::{
        frame_allocator::Frame,
        paging::{EntryFlags, MapToError, Page, PageTable, PAGE_SIZE},
    },
    scheduler, SCHEDULER,
};

/// helper function to work with `name` in Process
#[inline]
fn trim_trailing_zeros(slice: &[u8]) -> &[u8] {
    if let Some(last_non_zero) = slice.iter().rposition(|&x| x != 0) {
        &slice[..=last_non_zero]
    } else {
        &[]
    }
}

/// allocates and maps an area starting from `$start` with size `$size` and returns `Result<(), MapToError>` in `$page_table`
macro_rules! alloc_map {
    ($page_table: expr, $start: ident, $size: ident) => {
        let page_table = $page_table;

        const PAGES: usize = $size / PAGE_SIZE;
        const END: usize = $start + $size;

        // allocating frames
        let mut frames: [Frame; PAGES] = [Frame::containing_address(0); PAGES];

        for i in 0..frames.len() {
            frames[i] = $crate::memory::frame_allocator::allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;
        }

        for frame in frames {
            let virt_addr = frame.start_address | crate::hddm();
            let byte_array = virt_addr as *mut u8;
            let byte_array = unsafe { core::slice::from_raw_parts_mut(byte_array, PAGE_SIZE) };
            byte_array.fill(0);
        }

        let start_page = Page::containing_address($start);
        let end_page = Page::containing_address(END);

        let iter = Page::iter_pages(start_page, end_page);

        for (i, page) in iter.enumerate() {
            page_table.map_to(
                page,
                frames[i],
                EntryFlags::WRITABLE | EntryFlags::USER_ACCESSIBLE | EntryFlags::PRESENT,
            )?;
        }

        return Ok(());
    };
}

/// allocates and maps a stack to page_table
pub fn alloc_stack(page_table: &mut PageTable) -> Result<(), MapToError> {
    alloc_map!(page_table, STACK_START, STACK_SIZE);
}

/// allocates and maps the argv area to `page_table`
pub fn alloc_argv(page_table: &mut PageTable) -> Result<(), MapToError> {
    alloc_map!(page_table, ARGV_START, ARGV_SIZE);
}

/// allocates and maps a ring0 stack to page_table
pub fn alloc_ring0_stack(page_table: &mut PageTable) -> Result<(), MapToError> {
    alloc_map!(page_table, RING0_STACK_START, STACK_SIZE);
}
pub struct Scheduler {
    pub head: Box<Process>,
    /// raw pointers for peformance, we are ring0 we need the lowest stuff
    pub current_process: *mut Process,
    pub next_pid: u64,
    pub processes_count: usize,
}

impl Scheduler {
    #[inline]
    pub fn current_process(&self) -> &mut Process {
        unsafe { &mut *self.current_process }
    }

    #[inline]
    /// inits the scheduler
    /// jumps to `function` after initing!
    pub unsafe fn init(function: usize, name: &str) {
        debug!(Scheduler, "initing ...");
        asm!("cli");

        let mut process =
            Box::new(Process::new(function, 0, 0, name, &[], ProcessFlags::empty()).unwrap());

        let this = Self {
            current_process: &mut *process,
            head: process,
            next_pid: 1,
            processes_count: 1,
        };
        unsafe {
            (*SCHEDULER.0.get()) = MaybeUninit::new(this);
        }

        let context = scheduler().current_process().context;
        restore_cpu_status(&context)
    }

    /// context switches into next process, takes current context outputs new context
    pub unsafe fn switch(&mut self, context: CPUStatus) -> CPUStatus {
        unsafe { asm!("cli") }

        self.current_process().context = context;

        if self.current_process().status != ProcessStatus::WaitingForBurying {
            self.current_process().status = ProcessStatus::Waiting;
        }

        loop {
            if self
                .current_process()
                .next
                .as_ref()
                .is_some_and(|x| x.status == ProcessStatus::WaitingForBurying)
            {
                self.current_process().next = self.current_process().next.as_mut().unwrap().free();
                self.processes_count -= 1;
            }

            if self.current_process().next.is_some() {
                self.current_process = &mut **(*self.current_process).next.as_mut().unwrap();
            } else {
                self.current_process = &mut *self.head;
            }

            if self.current_process().status == ProcessStatus::Waiting {
                (*self.current_process).status = ProcessStatus::Running;
                break;
            }
        }

        return (*self.current_process).context;
    }

    /// appends a process to the end of the scheduler head
    pub fn add_process(&mut self, process: Process) {
        let mut current = &mut *self.head;
        while let Some(ref mut process) = current.next {
            current = &mut **process;
        }

        current.next = Some(Box::new(process));
        self.processes_count += 1;
    }

    pub fn find(&mut self, pid: u64) -> Option<&mut Process> {
        let mut current = &mut *self.head;
        if current.pid == pid {
            return Some(current);
        }

        let mut found = None;
        while let Some(ref mut process) = current.next {
            if process.pid == pid {
                found = Some(&mut **process);
                break;
            }

            current = &mut **process;
        }

        found
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
                expose::pkill(pid)?
            }

            Ok(())
        }
    }

    #[inline]
    pub fn set_next_resource(&mut self, next_ri: usize) {
        if next_ri < self.current_process().next_ri {
            self.current_process().next_ri = next_ri;
        }
    }

    #[inline]
    /// adds a resource to current process and returns it's ri
    // FIXME: this may become a little of a problem in multiple threads, the ri may be incorrect
    // if you add a resource while another is being added?
    // maybe we need a resource manager
    // system checklist
    // - Sync
    pub fn add_resource(&mut self, resource: processes::Resource) -> usize {
        let resources = &mut self.current_process().resources[self.current_process().next_ri..];

        for (mut ri, res) in resources.iter_mut().enumerate() {
            if res.variant() == Resource::Null.variant() {
                ri += self.current_process().next_ri;

                self.current_process().next_ri = ri;
                *res = resource;

                return ri;
            }
        }

        self.current_process().resources.push(resource);

        let ri = self.current_process().resources.len() - 1;
        self.current_process().next_ri = ri;

        return ri;
    }

    #[inline]
    pub fn remove_resource(&mut self, ri: usize) -> Result<(), ()> {
        if ri >= self.current_process().resources.len() {
            return Err(());
        }

        self.current_process().resources[ri] = Resource::Null;
        self.set_next_resource(ri);
        Ok(())
    }
}
