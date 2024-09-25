use core::slice;

use super::{ARGV_START, STACK_END};

use crate::drivers::vfs::expose::DirIter;
use crate::drivers::vfs::{vfs, FileDescriptor, FS};
use crate::memory::align_up;
use crate::utils::elf::{Elf, ElfError};
use crate::{arch, debug, kernel, scheduler, terminal, VirtAddr};

use crate::memory::paging::{self, EntryFlags, MapToError, Page, PAGE_SIZE};
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
}

impl Clone for Resource {
    fn clone(&self) -> Self {
        match self {
            Self::Null => Self::Null,
            Self::File(ref fd) => Self::File(fd.clone()),
            Self::DirIter(ref diriter) => Self::DirIter(DirIter::clone(&**diriter)),
        }
    }
}
impl Resource {
    pub const fn variant(&self) -> u8 {
        match self {
            Resource::Null => 0,
            Resource::File(_) => 1,
            Resource::DirIter(_) => 2,
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

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStatus {
    Waiting,
    Running,
    WaitingForBurying,
}

pub struct Process {
    pub ppid: u64,
    pub pid: u64,
    pub name: [u8; 64],
    pub status: ProcessStatus,
    pub context: CPUStatus,

    pub root_page_table: *mut PageTable,
    pub resources: Vec<Resource>,
    pub next_ri: usize,

    pub current_dir: String,

    data_pages: usize,
    pub data_break: usize,
    data_start: usize,
    pub next: Option<Box<Self>>,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct ProcessInfo {
    pub ppid: u64,
    pub pid: u64,
    pub name: [u8; 64],
    pub status: ProcessStatus,
}

#[inline]
fn copy_to_userspace(page_table: &mut PageTable, addr: VirtAddr, obj: &[u8]) {
    // FIXME: this assumes the next pages is mapped to the next frames
    // spilt obj into pages...
    let page = Page::containing_address(addr);
    let diff = addr - page.start_address;

    let frame = page_table.get_frame(page).unwrap();

    let phys_addr = frame.start_address + diff;
    let virt_addr = phys_addr | kernel().phy_offset;
    unsafe {
        core::ptr::copy_nonoverlapping(obj.as_ptr(), virt_addr as *mut u8, obj.len());
    }
}
impl Process {
    #[inline(always)]
    pub fn new(
        function: usize,
        ppid: u64,
        pid: u64,
        name: &str,
        argv: &[&str],
        flags: ProcessFlags,
    ) -> Result<Self, MapToError> {
        let name_bytes = name.as_bytes();

        let mut name = [0u8; 64];

        let len = name_bytes.len().min(64);
        name[..len].copy_from_slice(&name_bytes[..len]);

        let status = ProcessStatus::Waiting;
        let mut context = CPUStatus::default();

        let root_page_table_addr = paging::allocate_pml4()?;
        let root_page_table = (root_page_table_addr | kernel().phy_offset) as *mut PageTable;

        unsafe {
            let page_table = &mut *root_page_table;
            super::alloc_stack(page_table)?;
            super::alloc_argv(page_table)?;

            if argv.len() != 0 {
                let mut start_addr = ARGV_START;
                const USIZE_BYTES: usize = size_of::<usize>();
                let argc = argv.len();

                // argc
                copy_to_userspace(
                    page_table,
                    start_addr,
                    &core::mem::transmute::<_, [u8; USIZE_BYTES]>(argc),
                );

                // argv*
                start_addr += USIZE_BYTES;

                for arg in argv {
                    let arg = arg.as_bytes();
                    let len = arg.len();

                    copy_to_userspace(
                        page_table,
                        start_addr,
                        &core::mem::transmute::<_, [u8; USIZE_BYTES]>(len),
                    );
                    start_addr += USIZE_BYTES;

                    copy_to_userspace(page_table, start_addr, arg);
                    start_addr += len;
                }

                let argv_addr = start_addr;
                let mut current_argv_ptr = ARGV_START + USIZE_BYTES /* after argc */;
                // argv**
                for arg in argv {
                    copy_to_userspace(
                        page_table,
                        start_addr,
                        &core::mem::transmute::<_, [u8; USIZE_BYTES]>(current_argv_ptr),
                    );
                    start_addr += USIZE_BYTES;

                    current_argv_ptr += USIZE_BYTES; // skip the len
                    current_argv_ptr += arg.len(); // skip the data
                }

                // set rdi and rsi to argc and argv
                // _start looks like: extern "C" _start(argc: u64, argv: *const &str)
                #[cfg(target_arch = "x86_64")]
                {
                    context.rdi = argc as u64;
                    context.rsi = argv_addr as u64;
                }
                // looks like this: argc: 8 (u64) -> argv: (len: 8 (u64) + bytes: len ([u8])) * argc -> argv_pointers: 8 (u64) * argc
                // where numbers is bytes count, (TYPE) is the type of the bytes
            }
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
        // FIXME: the code isn't prepared for any null bs fixme!!!!!!!!!!!!!!!
        // stdin fd
        resources.push(Resource::File(FileDescriptor {
            mountpoint: terminal(),
            node: core::ptr::null_mut(),
            read_pos: 0,
            write_pos: 0,
        }));
        // stdout fd
        resources.push(Resource::File(FileDescriptor {
            mountpoint: terminal(),
            node: core::ptr::null_mut(),
            read_pos: 0,
            write_pos: 0,
        }));

        Ok(Process {
            ppid,
            pid,
            name,
            status,
            context,

            root_page_table,
            resources,
            current_dir: String::from("ram:/"),
            next_ri: 0,

            data_pages: 0,
            data_break: 0,
            data_start: 0,
            next: None,
        })
    }

    pub fn create(
        function: usize,
        name: &str,
        argv: &[&str],
        flags: ProcessFlags,
    ) -> Result<Self, MapToError> {
        let pid = scheduler().next_pid;
        debug!(
            Process,
            "creating a process with pid {} ({}) ...", pid, name
        );

        let results = Self::new(
            function,
            scheduler().current_process().pid,
            pid,
            name,
            argv,
            flags,
        )?;
        scheduler().next_pid += 1;

        debug!(Process, "success ...");
        Ok(results)
    }

    #[inline(always)]
    /// creates a userspace process from an elf
    pub fn from_elf(elf: Elf, name: &str, argv: &[&str]) -> Result<Self, ElfError> {
        let mut process = Self::create(elf.header.entry_point, name, argv, ProcessFlags::USERSPACE)
            .ok()
            .ok_or(ElfError::MapToError)?;
        let data_break = unsafe { elf.load_exec(&mut *process.root_page_table)? };

        process.data_start = align_up(data_break, PAGE_SIZE);
        process.data_break = process.data_start;
        Ok(process)
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

    pub fn info(&self) -> ProcessInfo {
        ProcessInfo {
            ppid: self.ppid,
            pid: self.pid,
            name: self.name,
            status: self.status,
        }
    }

    #[inline(always)]
    fn data_break_actual(&self) -> usize {
        self.data_start + PAGE_SIZE * self.data_pages
    }

    fn page_extend_data(&mut self) -> Result<(), MapToError> {
        let page_end = self.data_break_actual();
        let new_page = Page::containing_address(page_end);

        let frame = kernel()
            .frame_allocator()
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        unsafe {
            (*self.root_page_table).map_to(
                new_page,
                frame,
                EntryFlags::WRITABLE | EntryFlags::USER_ACCESSIBLE | EntryFlags::PRESENT,
            )?
        };

        let addr = frame.start_address | kernel().phy_offset;
        let ptr = addr as *mut u8;
        let slice = unsafe { slice::from_raw_parts_mut(ptr, PAGE_SIZE) };

        slice.fill(0);
        self.data_pages += 1;
        Ok(())
    }

    pub fn extend_data_by(&mut self, amount: usize) -> Result<*mut u8, MapToError> {
        while self.data_break_actual() < self.data_break + amount {
            self.page_extend_data()?;
        }

        self.data_break += amount;
        Ok(self.data_break as *mut u8)
    }
}
