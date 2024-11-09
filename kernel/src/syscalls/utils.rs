use crate::{
    threading::{self, expose::ErrorStatus},
    utils::{self, expose::SysInfo},
};

use super::{make_slice, make_slice_mut};

/// for now
#[no_mangle]
extern "C" fn sysexit(code: usize) {
    threading::expose::thread_exit(code);
}

#[no_mangle]
extern "C" fn sysyield() {
    threading::expose::thread_yeild()
}

#[no_mangle]
extern "C" fn syschdir(path_ptr: *const u8, path_len: usize) -> ErrorStatus {
    let slice = make_slice!(path_ptr, path_len);
    let Ok(name) = core::str::from_utf8(slice) else {
        return ErrorStatus::InvaildStr;
    };

    if let Err(err) = threading::expose::chdir(name) {
        err.into()
    } else {
        ErrorStatus::None
    }
}

#[no_mangle]
extern "C" fn sysgetcwd(path_ptr: *mut u8, len: usize, dest_len: *mut usize) -> ErrorStatus {
    let slice = make_slice_mut!(path_ptr, len);
    let got = threading::expose::getcwd().as_bytes();

    if got.len() > len {
        return ErrorStatus::Generic;
    }

    slice[..got.len()].copy_from_slice(&got);

    if !dest_len.is_null() {
        unsafe {
            *dest_len = got.len();
        }
    }

    ErrorStatus::None
}

// on fail returns null for unknown reasons
#[no_mangle]
extern "C" fn syssbrk(amount: isize) -> *mut u8 {
    threading::expose::sbrk(amount)
}

#[no_mangle]
extern "C" fn sysinfo(ptr: *mut SysInfo) -> ErrorStatus {
    if ptr.is_null() || !ptr.is_aligned() {
        return ErrorStatus::InvaildPtr;
    }

    unsafe {
        utils::expose::info(&mut *ptr);
    }

    ErrorStatus::None
}
