use core::arch::asm;

use alloc::{boxed::Box, vec::Vec};
use bitflags::bitflags;

use crate::{
    arch::{
        self,
        threading::{restore_cpu_status, CPUStatus},
    },
    kernel,
    memory::{
        frame_allocator::Frame,
        paging::{allocate_pml4, EntryFlags, MapToError, Page, PageTable, PAGE_SIZE},
    },
    scheduler, serial, SCHEDULER,
};

bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct ProcessFlags: u8 {
        const USERSPACE = 1 << 0;
    }
}

pub const STACK_SIZE: usize = PAGE_SIZE * 4;
pub const STACK_START: usize = 0x00007A0000000000;
pub const STACK_END: usize = STACK_START + STACK_SIZE;

/// helper function to work with `name` in Process
#[inline]
fn trim_trailing_zeros(slice: &[u8]) -> &[u8] {
    if let Some(last_non_zero) = slice.iter().rposition(|&x| x != 0) {
        &slice[..=last_non_zero]
    } else {
        &[]
    }
}

/// allocates and maps a stack to page_table
pub fn alloc_stack(page_table: &mut PageTable) -> Result<(), MapToError> {
    // allocating frames
    let mut frames: [Frame; STACK_SIZE / PAGE_SIZE] =
        [Frame::containing_address(0); STACK_SIZE / PAGE_SIZE];

    for i in 0..frames.len() {
        frames[i] = kernel()
            .frame_allocator()
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
    }

    for frame in frames {
        let virt_addr = frame.start_address | kernel().phy_offset;
        let byte_array = virt_addr as *mut u8;
        let byte_array = unsafe { core::slice::from_raw_parts_mut(byte_array, PAGE_SIZE) };
        byte_array.fill(0);
    }

    let start_page = Page::containing_address(STACK_START);
    let end_page = Page::containing_address(STACK_END); // === STACK_END

    let iter = Page::iter_pages(start_page, end_page);

    for (i, page) in iter.enumerate() {
        page_table.map_to(
            page,
            frames[i],
            EntryFlags::WRITABLE | EntryFlags::USER_ACCESSIBLE | EntryFlags::PRESENT,
        )?;
    }

    Ok(())
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
    pub next: Option<Box<Self>>,
}

impl Process {
    #[inline]
    pub fn new(function: usize, pid: u64, name: &str, flags: ProcessFlags) -> Self {
        let name_bytes = name.as_bytes();

        let mut name = [0u8; 64];

        let len = name_bytes.len().min(64);
        name[..len].copy_from_slice(&name_bytes[..len]);

        let status = ProcessStatus::Waiting;
        let mut context = CPUStatus::default();

        let root_page_table_addr = allocate_pml4().unwrap();
        let root_page_table = (root_page_table_addr | kernel().phy_offset) as *mut PageTable;

        unsafe {
            alloc_stack(&mut *root_page_table).unwrap();
        }

        #[cfg(target_arch = "x86_64")]
        {
            use arch::x86_64::threading::RFLAGS;

            context.rsp = STACK_END as u64;
            context.rip = function as u64;

            // Kernel process
            if flags.is_empty() {
                context.rflags = RFLAGS::from_bits_retain(0x202);

                context.ss = arch::x86_64::gdt::KERNEL_DATA_SEG as u64;
                context.cs = arch::x86_64::gdt::KERNEL_CODE_SEG as u64;
            } else if flags.contains(ProcessFlags::USERSPACE) {
                context.rflags = RFLAGS::IOPL_LOW
                    | RFLAGS::IOPL_HIGH
                    | RFLAGS::INTERRUPT_FLAG
                    | RFLAGS::from_bits_retain(0x2);

                context.ss = arch::x86_64::gdt::USER_DATA_SEG as u64;
                context.cs = arch::x86_64::gdt::USER_CODE_SEG as u64;
            }
            context.cr3 = root_page_table_addr as u64;
        }

        Process {
            pid,
            name,
            status,
            context,

            root_page_table,
            next: None,
        }
    }

    pub fn create(function: usize, name: &str, flags: ProcessFlags) -> Self {
        let pid = scheduler().next_pid;
        let results = Self::new(function, pid, name, flags);
        scheduler().next_pid += 1;
        results
    }

    /// frees self and then returns next
    /// frees all resources that has something to do with this process even the process stack and
    /// page table
    /// TODO: test this properly
    pub fn free(&mut self) -> Option<Box<Process>> {
        serial!("deallocating a process! ...\n");

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
    pub next_pid: u64,
}

impl Scheduler {
    #[inline]
    /// inits the scheduler
    /// jumps to `function` after initing!
    pub unsafe fn init(function: usize, name: &str) {
        serial!("initing the scheduler...\n");
        asm!("cli");

        let mut process = Box::new(Process::new(function, 0, name, ProcessFlags::empty()));

        let this = Self {
            current_process: &mut *process,
            head: process,
            next_pid: 1,
        };

        SCHEDULER = Some(this);

        let context = (*scheduler().current_process).context;
        //
        // serial!("restoring...!\n");
        // unsafe { asm!("mov cr3, rax; mov rsp, rcx", in("rax") context.cr3, in("rcx") context.rsp, options()) }
        //
        // let context = (*scheduler().current_process).context;
        //
        // unsafe {
        //     asm!(
        //         "
        //         mov rbp, 0
        //         mov cr3, {}
        //         push {}
        //         push {}
        //         push {}
        //         push {}
        //         push {}
        //         iretq
        //         ", in(reg) context.cr3, in(reg) context.ss, in(reg) context.rsp, in(reg) context.rflags.bits(), in(reg) context.cs, in(reg) context.rip
        //     )
        // }
        //
        restore_cpu_status(&context)
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
    pub fn add_process(&mut self, process: Process) {
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
    pub fn create_process(&mut self, function: usize, name: &str, flags: ProcessFlags) {
        self.add_process(Process::create(function, name, flags));
    }
}
