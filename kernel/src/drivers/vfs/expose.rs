use core::{mem::MaybeUninit, usize};

use alloc::{boxed::Box, string::ToString};

use crate::{scheduler, threading::processes::Resource};

use super::{vfs, FSError, FSResult, Inode, InodeType, Path, FS};
/// gets a FileDescriptor from a fd (file_descriptor id) may return Err(FSError::InvaildFileDescriptor)
macro_rules! get_fd {
    ($ri: expr) => {{
        let Some(crate::threading::processes::Resource::File(ref mut file_descriptor)) =
            crate::scheduler().resources().get_mut($ri)
        else {
            return Err(FSError::InvaildFileDescriptor);
        };

        file_descriptor
    }};
}

/// ffi safe file info
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FileDescriptorStat {
    // TODO: this is unsafe replace with inode_id
    pub inode: *mut Inode,
    pub kind: InodeType,
    pub size: usize,
}

impl FileDescriptorStat {
    /// returns a null file descriptor
    /// it is all zeros
    #[inline]
    pub unsafe fn default() -> Self {
        core::mem::zeroed()
    }
    /// the file name length in bytes you can then grab the file name using
    /// FileDescriptorStat::get_name
    pub fn name_length(&self) -> usize {
        unsafe { (*self.inode).name.len() }
    }

    pub fn get_name(&self, buffer: &mut [u8]) -> FSResult<()> {
        let name = unsafe { &(*self.inode).name };
        if buffer.len() != name.len() {
            return Err(FSError::InvaildBuffer);
        }

        buffer.copy_from_slice(name.as_bytes());
        Ok(())
    }

    /// lives as long as fd is opened
    /// the only thing that is going to get UD'ed is the `self.name`
    pub fn get(ri: usize, stat: &mut Self) -> FSResult<()> {
        let file_descriptor = get_fd!(ri);
        let kind = unsafe { (*file_descriptor.node).inode_type };

        *stat = Self {
            inode: file_descriptor.node,
            kind,
            size: file_descriptor.size(),
        };

        Ok(())
    }
}

#[no_mangle]
pub fn open(path: Path) -> FSResult<usize> {
    let fd = vfs().open(path)?;
    scheduler().resources().push(Resource::File(fd));
    Ok(scheduler().resources().len() - 1)
}

#[no_mangle]
pub fn close(ri: usize) -> FSResult<()> {
    _ = ri;
    todo!()
}

#[no_mangle]
pub fn read(ri: usize, buffer: &mut [u8]) -> FSResult<usize> {
    let fd = get_fd!(ri);
    vfs().read(fd, buffer)
}

#[no_mangle]
pub fn write(ri: usize, buffer: &[u8]) -> FSResult<()> {
    let fd = get_fd!(ri);
    vfs().write(fd, buffer)
}

#[no_mangle]
pub fn create(path: Path, name: &str) -> FSResult<()> {
    vfs().create(path, name.to_string())
}

#[no_mangle]
pub fn createdir(path: Path, name: &str) -> FSResult<()> {
    vfs().createdir(path, name.to_string())
}

pub const MAX_NAME_LEN: usize = 128;

#[derive(Debug, PartialEq, Eq)]
#[repr(C)]
pub struct DirEntry {
    pub kind: InodeType,
    pub size: usize,
    pub name_length: usize,
    pub name: [u8; 128],
}

impl DirEntry {
    pub fn get_from_inode(inode: *const Inode) -> FSResult<Self> {
        unsafe {
            let kind = (*inode).inode_type;
            let size = (*inode).size();
            let name_slice = (*inode).name.as_bytes();

            let name_length = name_slice.len();
            let mut name = [0u8; MAX_NAME_LEN];

            name[..name_length].copy_from_slice(name_slice);

            Ok(Self {
                kind,
                size,
                name_length,
                name,
            })
        }
    }

    pub const unsafe fn zeroed() -> Self {
        core::mem::zeroed()
    }
}

#[repr(C)]
pub struct DirIterS {
    diriter: MaybeUninit<*mut dyn DirIter>,
}
impl DirIterS {
    pub const unsafe fn zeroed() -> Self {
        #[allow(invalid_value)]
        core::mem::zeroed()
    }
}
pub trait DirIter: Iterator<Item = DirEntry> {}

#[no_mangle]
pub fn diriter_open(ri: usize, diriter_s: &mut DirIterS) -> FSResult<()> {
    let fd = get_fd!(ri);
    let diriter = vfs().diriter_open(fd)?;

    *diriter_s = DirIterS {
        diriter: MaybeUninit::new(Box::into_raw(diriter)),
    };
    Ok(())
}

pub fn diriter_next(diriter_s: &mut DirIterS, direntry: &mut DirEntry) {
    let next = unsafe { (*diriter_s.diriter.assume_init()).next() };
    if let Some(entry) = next {
        *direntry = entry;
        return;
    }

    unsafe { *direntry = DirEntry::zeroed() };
}

#[no_mangle]
pub fn diriter_close(diriter_s: &mut DirIterS) {
    let boxed = unsafe { Box::from_raw(diriter_s.diriter.assume_init()) };
    drop(boxed);
}
