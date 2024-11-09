use alloc::string::String;

use crate::{
    make_slice, make_slice_mut,
    threading::{
        self,
        expose::{ErrorStatus, SpawnFlags},
        processes::ProcessInfo,
    },
};

#[no_mangle]
extern "C" fn syswait(pid: u64) -> usize {
    threading::expose::wait(pid)
}

// argv can be null
// name can be null but the spawned process name will be different in case of spawn or pspawn
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SpawnConfig {
    pub name_ptr: *const u8,
    pub name_len: usize,
    pub argv: *mut (*const u8, usize),
    pub argc: usize,
    pub flags: SpawnFlags,
}

// if dest_pid is null we will just ignore it
#[no_mangle]
extern "C" fn sysspawn(
    elf_ptr: *const u8,
    elf_len: usize,
    config: *const SpawnConfig,
    dest_pid: *mut u64,
) -> ErrorStatus {
    let (name_ptr, name_len, argc, argv, flags) = unsafe {
        let config = *config;
        (
            config.name_ptr,
            config.name_len,
            config.argc,
            config.argv,
            config.flags,
        )
    };

    let name = if !name_ptr.is_null() {
        make_slice!(name_ptr, name_len)
    } else {
        &[]
    };
    let name = String::from_utf8_lossy(name);

    let argv = if !argv.is_null() {
        make_slice_mut!(argv, argc)
    } else {
        &mut []
    };

    let argv_str: &mut [&str] = unsafe { core::mem::transmute(&mut *argv) };

    for (i, arg) in argv.iter().enumerate() {
        // transmut doesn't work we make it work here
        let slice = make_slice!(arg.0, arg.1);
        unsafe {
            argv_str[i] = core::str::from_utf8_unchecked(slice);
        }
        // argv[i] is invaild after this
        // argv_str[i] is argv[i] but in a rusty way
    }

    let elf_bytes = make_slice!(elf_ptr, elf_len);
    unsafe {
        match threading::expose::spawn(&name, elf_bytes, argv_str, flags) {
            Err(err) => err.into(),
            Ok(pid) => {
                if !dest_pid.is_null() {
                    *dest_pid = pid
                }
                ErrorStatus::None
            }
        }
    }
}

#[no_mangle]
extern "C" fn syspcollect(ptr: *mut ProcessInfo, len: usize) -> ErrorStatus {
    let slice = make_slice_mut!(ptr, len);

    if let Err(()) = threading::expose::pcollect(slice) {
        ErrorStatus::Generic
    } else {
        ErrorStatus::None
    }
}
