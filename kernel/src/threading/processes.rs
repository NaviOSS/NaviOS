use super::STACK_END;

use crate::drivers::vfs::expose::DirIter;
use crate::drivers::vfs::{vfs, FileDescriptor, FS};
use crate::{arch, debug, kernel, scheduler};

use crate::memory::paging;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use bitflags::bitflags;

use crate::{arch::threading::CPUStatus, memory::paging::PageTable};

#[derive(Debug)]
pub enum Resource {
    Null,
    File(FileDescriptor),
    DirIter(Box<dyn DirIter>),
    /// FIXME:
    /// dirty soultion it is basically a File that points to a kernel-device the thing is i dont
    /// have devices yet
    Reserved,
}

impl Resource {
    pub const fn variant(&self) -> u8 {
        match self {
            Resource::Null => 0,
            Resource::File(_) => 1,
            Resource::DirIter(_) => 2,
            Resource::Reserved => 3,
        }
    }
}

bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct ProcessFlags: u8 {
        const USERSPACE = 1 << 0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStatus {
    Waiting,
    Running,
    WaitingForBurying,
}

pub struct Process {
    pub pid: u64,
    pub name: [u8; 64],
    pub status: ProcessStatus,
    pub context: CPUStatus,

    pub root_page_table: *mut PageTable,
    pub resources: Vec<Resource>,
    pub next_ri: usize,

    pub current_dir: String,
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

        let root_page_table_addr = paging::allocate_pml4().unwrap();
        let root_page_table = (root_page_table_addr | kernel().phy_offset) as *mut PageTable;

        unsafe {
            super::alloc_stack(&mut *root_page_table).unwrap();
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

        let mut resources = Vec::with_capacity(2);
        // stdin fd
        resources.push(Resource::Reserved);
        // stdout fd
        resources.push(Resource::Reserved);

        Process {
            pid,
            name,
            status,
            context,

            root_page_table,
            resources,
            current_dir: String::from("ram:/"),
            next_ri: 0,
            next: None,
        }
    }

    pub fn create(function: usize, name: &str, flags: ProcessFlags) -> Self {
        let pid = scheduler().next_pid;
        debug!(
            Process,
            "creating a process with pid {} ({}) ...", pid, name
        );

        let results = Self::new(function, pid, name, flags);
        scheduler().next_pid += 1;

        debug!(Process, "success ...");
        results
    }

    /// frees self and then returns next
    /// frees all resources that has something to do with this process and all it's memory
    pub fn free(&mut self) -> Option<Box<Process>> {
        debug!(Process, "deallocating a process with pid {} ...", self.pid);

        let root_page_table = unsafe { &mut (*self.root_page_table) };
        unsafe { root_page_table.free(4) };

        debug!(Process, "deallocated the process's page table ...");

        for resource in &mut self.resources {
            match resource {
                Resource::File(ref mut fd) => vfs().close(fd).unwrap(),
                _ => (),
            }
        }

        debug!(Process, "closed process resources ...");

        self.next.take()
    }
}
