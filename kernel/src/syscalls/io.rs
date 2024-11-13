use crate::{
    drivers::vfs::{self, expose::open, FSError},
    threading,
    utils::{
        errors::ErrorStatus,
        ffi::{Optional, Slice, SliceMut},
    },
};

#[no_mangle]
extern "C" fn sysopen(path_ptr: *const u8, len: usize, dest_fd: Optional<usize>) -> ErrorStatus {
    let path = Slice::new(path_ptr, len)?.into_str();

    match open(path) {
        Ok(fd) => {
            if let Some(dest_fd) = dest_fd.into_option() {
                *dest_fd = fd;
            }
            ErrorStatus::None
        }
        Err(err) => err.into(),
    }
}

#[no_mangle]
extern "C" fn syswrite(fd: usize, ptr: *const u8, len: usize) -> ErrorStatus {
    let slice = Slice::new(ptr, len)?.into_slice();
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
extern "C" fn sysread(
    fd: usize,
    ptr: *mut u8,
    len: usize,
    dest_read: Optional<usize>,
) -> ErrorStatus {
    let slice = SliceMut::new(ptr, len)?.into_slice();

    loop {
        match vfs::expose::read(fd, slice) {
            Err(FSError::ResourceBusy) => threading::expose::thread_yeild(),
            Err(err) => return err.into(),
            Ok(bytes_read) => {
                if let Some(dest_read) = dest_read.into_option() {
                    *dest_read = bytes_read;
                }
                return ErrorStatus::None;
            }
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
    let path = Slice::new(path_ptr, path_len)?.into_str();

    if let Err(err) = vfs::expose::create(path) {
        err.into()
    } else {
        ErrorStatus::None
    }
}

#[no_mangle]
extern "C" fn syscreatedir(path_ptr: *const u8, path_len: usize) -> ErrorStatus {
    let path = Slice::new(path_ptr, path_len)?.into_str();

    if let Err(err) = vfs::expose::createdir(path) {
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
    match vfs::expose::diriter_next(diriter_ri, &mut *direntry) {
        Err(err) => err.into(),
        Ok(()) => ErrorStatus::None,
    }
}

#[no_mangle]
extern "C" fn sysfstat(ri: usize, direntry: *mut vfs::expose::DirEntry) -> ErrorStatus {
    unsafe {
        if let Err(err) = vfs::expose::fstat(ri, &mut *direntry) {
            err.into()
        } else {
            ErrorStatus::None
        }
    }
}
