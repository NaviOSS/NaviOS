use core::{cell::UnsafeCell, mem::MaybeUninit};

use lazy_static::lazy_static;
use spin::Mutex;

use crate::{
    limine,
    memory::buddy_allocator::BuddyAllocator,
    threading::Scheduler,
    utils::{self, elf::Elf, Locked},
};
// TODO: figure out a safer way to Scheduler
pub struct SchedulerWrapper(pub UnsafeCell<MaybeUninit<Scheduler>>);

unsafe impl Sync for SchedulerWrapper {}
pub static SCHEDULER: SchedulerWrapper = SchedulerWrapper(UnsafeCell::new(MaybeUninit::zeroed()));

pub fn scheduler_inited() -> bool {
    scheduler().current_process != core::ptr::null_mut()
}
#[inline(always)]
pub fn scheduler() -> &'static mut Scheduler {
    unsafe { (*SCHEDULER.0.get()).assume_init_mut() }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: Locked<MaybeUninit<BuddyAllocator>> =
    unsafe { Locked::new(BuddyAllocator::new()) };

pub fn global_allocator() -> &'static Mutex<MaybeUninit<BuddyAllocator<'static>>> {
    &GLOBAL_ALLOCATOR.inner
}
/// static mut because we need really fast access of HDDM
pub static mut HDDM: usize = 0;
#[inline(always)]
pub fn hddm() -> usize {
    unsafe { HDDM }
}

lazy_static! {
    pub static ref KERNEL_ELF: Elf<'static> = {
        let kernel_img = limine::kernel_image_info();
        let kernel_img_bytes = unsafe { core::slice::from_raw_parts(kernel_img.0, kernel_img.1) };
        let elf = utils::elf::Elf::new(kernel_img_bytes).unwrap();
        elf
    };
    pub static ref RSDP_ADDR: usize = limine::rsdp_addr();
}
