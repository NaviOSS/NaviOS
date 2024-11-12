use alloc::{string::String, vec::Vec};

use crate::{
    drivers::vfs::{
        expose::{fstat, open, read, DirEntry},
        InodeType,
    },
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
/// spawns an elf process from a path
/// this is a little bit of a hack for now
fn pspawn(path: &str, config: *const SpawnConfig, dest_pid: *mut u64) -> Result<(), ErrorStatus> {
    let file = open(path).map_err(|e| e.into())?;

    let mut stat = unsafe { DirEntry::zeroed() };
    fstat(file, &mut stat).map_err(|e| e.into())?;

    if stat.kind != InodeType::File {
        return Err(ErrorStatus::NotAFile);
    }
    let mut buffer = Vec::with_capacity(stat.size);
    buffer.resize(stat.size, 0);
    read(file, &mut buffer).map_err(|e| e.into())?;
    Err(sysspawn(buffer.as_ptr(), buffer.len(), config, dest_pid))
}
#[no_mangle]
extern "C" fn syspspawn(
    path: *const u8,
    len: usize,
    config: *const SpawnConfig,
    dest_pid: *mut u64,
) -> ErrorStatus {
    let path = make_slice!(path, len);
    let path = unsafe { core::str::from_utf8_unchecked(path) };
    unsafe {
        pspawn(path, config, dest_pid)
            .map_err(|e| e.into())
            .unwrap_err_unchecked()
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
