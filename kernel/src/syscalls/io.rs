use alloc::string::String;

use crate::{
    drivers::vfs::{self, expose::open, FSError},
    threading::{self, expose::ErrorStatus},
};

use super::{make_slice, make_slice_mut};
#[no_mangle]
extern "C" fn sysopen(path_ptr: *const u8, len: usize, dest_fd: *mut usize) -> ErrorStatus {
    if dest_fd.is_null() {
        return ErrorStatus::InvaildPtr;
    }

    let path = make_slice!(path_ptr, len);

    let path = unsafe { core::str::from_utf8_unchecked(path) };

    match open(path) {
        Ok(fd) => unsafe {
            *dest_fd = fd;
            ErrorStatus::None
        },
        Err(err) => err.into(),
    }
}

#[no_mangle]
extern "C" fn syswrite(fd: usize, ptr: *const u8, len: usize) -> ErrorStatus {
    let slice = make_slice!(ptr, len);
    while let Err(err) = vfs::expose::write(fd, slice) {
        match err {
            FSError::ResourceBusy => {
                threading::expose::thread_yeild();
            }
            _ => return err.into(),
        }
    }
    ErrorStatus::None
}

#[no_mangle]
extern "C" fn sysread(fd: usize, ptr: *mut u8, len: usize, dest_read: *mut usize) -> ErrorStatus {
    let slice = make_slice_mut!(ptr, len);

    loop {
        match vfs::expose::read(fd, slice) {
            Err(FSError::ResourceBusy) => threading::expose::thread_yeild(),
            Err(err) => return err.into(),
            Ok(bytes_read) => unsafe {
                *dest_read = bytes_read;
                return ErrorStatus::None;
            },
        }
    }
}

#[no_mangle]
extern "C" fn sysclose(fd: usize) -> ErrorStatus {
    if let Err(err) = vfs::expose::close(fd) {
        err.into()
    } else {
        ErrorStatus::None
    }
}

#[no_mangle]
extern "C" fn syscreate(path_ptr: *const u8, path_len: usize) -> ErrorStatus {
    let path = make_slice!(path_ptr, path_len);
    let path = String::from_utf8_lossy(path);

    if let Err(err) = vfs::expose::create(&path) {
        err.into()
    } else {
        ErrorStatus::None
    }
}

#[no_mangle]
extern "C" fn syscreatedir(path_ptr: *const u8, path_len: usize) -> ErrorStatus {
    let path = make_slice!(path_ptr, path_len);
    let path = String::from_utf8_lossy(path);

    if let Err(err) = vfs::expose::createdir(&path) {
        err.into()
    } else {
        ErrorStatus::None
    }
}

#[no_mangle]
extern "C" fn sysdiriter_open(dir_ri: usize, dest_diriter: *mut usize) -> ErrorStatus {
    match vfs::expose::diriter_open(dir_ri) {
        Err(err) => err.into(),
        Ok(ri) => unsafe {
            *dest_diriter = ri;
            ErrorStatus::None
        },
    }
}

#[no_mangle]
extern "C" fn sysdiriter_close(diriter_ri: usize) -> ErrorStatus {
    match vfs::expose::diriter_close(diriter_ri) {
        Err(err) => err.into(),
        Ok(()) => ErrorStatus::None,
    }
}

#[no_mangle]
unsafe extern "C" fn sysdiriter_next(
    diriter_ri: usize,
    direntry: *mut vfs::expose::DirEntry,
) -> ErrorStatus {
    if direntry.is_null() || !direntry.is_aligned() {
        return ErrorStatus::InvaildPtr;
    }

    match vfs::expose::diriter_next(diriter_ri, &mut *direntry) {
        Err(err) => err.into(),
        Ok(()) => ErrorStatus::None,
    }
}

#[no_mangle]
extern "C" fn sysfstat(ri: usize, direntry: *mut vfs::expose::DirEntry) -> ErrorStatus {
    if direntry.is_null() || !direntry.is_aligned() {
        return ErrorStatus::InvaildPtr;
    }

    unsafe {
        if let Err(err) = vfs::expose::fstat(ri, &mut *direntry) {
            err.into()
        } else {
            ErrorStatus::None
        }
    }
}
