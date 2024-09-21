use crate::{kernel, limine::MEMORY_SIZE, memory::paging::PAGE_SIZE, scheduler};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SysInfo {
    pub total_mem: usize,
    pub used_mem: usize,
    pub processes_count: usize,
}

impl SysInfo {
    pub fn null() -> Self {
        Self {
            total_mem: 0,
            used_mem: 0,
            processes_count: 0,
        }
    }
}

#[no_mangle]
pub fn info(sysinfo: &mut SysInfo) {
    let mut used_mem = 0;

    for byte in &*kernel().frame_allocator().bitmap {
        for i in 0..8 {
            if (*byte >> i) & 1 == 1 {
                used_mem += PAGE_SIZE;
            }
        }
    }

    *sysinfo = SysInfo {
        total_mem: *MEMORY_SIZE,
        used_mem,
        processes_count: scheduler().processes_count,
    }
}
