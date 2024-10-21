use core::{cell::UnsafeCell, mem::MaybeUninit};

use spin::Mutex;

use crate::{
    memory::buddy_allocator::BuddyAllocator,
    threading::Scheduler,
    utils::{elf::Elf, Locked},
};

/// boot info
#[derive(Debug)]
pub struct Kernel<'a> {
    pub phy_offset: usize,
    pub rsdp_addr: Option<u64>,
    pub elf: Elf<'a>,

    pub scheduler: MaybeUninit<Scheduler>,
}

pub struct KernelWrapper(UnsafeCell<MaybeUninit<Kernel<'static>>>);
unsafe impl Sync for KernelWrapper {}

impl KernelWrapper {
    pub fn get(&self) -> &mut Kernel {
        unsafe { &mut *self.0.get().cast::<Kernel>() }
    }
    pub fn inited(&self) -> bool {
        self.get().phy_offset != 0
    }
}

pub static KERNEL: KernelWrapper = KernelWrapper(UnsafeCell::new(MaybeUninit::zeroed()));

pub fn kernel<'a>() -> &'a mut Kernel<'a> {
    KERNEL.get()
}
pub fn kernel_inited() -> bool {
    KERNEL.inited()
}

pub fn scheduler_inited() -> bool {
    unsafe { kernel().scheduler.assume_init_ref().current_process != core::ptr::null_mut() }
}

pub fn scheduler() -> &'static mut Scheduler {
    unsafe { kernel().scheduler.assume_init_mut() }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: Locked<MaybeUninit<BuddyAllocator>> =
    unsafe { Locked::new(BuddyAllocator::new()) };

pub fn global_allocator() -> &'static Mutex<MaybeUninit<BuddyAllocator<'static>>> {
    &GLOBAL_ALLOCATOR.inner
}
