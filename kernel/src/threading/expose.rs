use core::arch::asm;

use crate::{
    debug, khalt, scheduler,
    threading::processes::{Process, ProcessStatus},
};

#[no_mangle]
pub fn thread_exit() {
    scheduler().current_process().status = ProcessStatus::WaitingForBurying;
    // enables interrupts if they were disabled to give control back to the scheduler
    #[cfg(target_arch = "x86_64")]
    unsafe {
        asm!("sti")
    }
    khalt()
}

#[no_mangle]
pub fn thread_yeild() {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        asm!("int 0x20")
    }
}

#[no_mangle]
pub fn wait(pid: u64) {
    debug!(
        Process,
        "{} waiting for {} to exit ...",
        scheduler().current_process().pid,
        pid
    );

    loop {
        let mut current = scheduler().head.as_mut();
        let mut found = false;

        while let Some(ref mut process) = current.next {
            if process.pid == pid {
                found = true;
                if process.status == ProcessStatus::WaitingForBurying {
                    return;
                }
            }

            current = process;
            thread_yeild()
        }

        if !found {
            return;
        }

        thread_yeild()
    }
}
