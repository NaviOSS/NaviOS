//! exposed functions of VFS they manually uses
//! a resource index instead of a file descriptor aka ri
use core::{fmt::Debug, usize};

use alloc::boxed::Box;

use crate::{scheduler, threading::processes::Resource};

use super::{vfs, FSError, FSResult, Inode, InodeType, Path, FS};
/// gets a FileDescriptor from a fd (file_descriptor id) may return Err(FSError::InvaildFileDescriptor)
macro_rules! get_fd {
    ($ri: expr) => {{
        let Some(crate::threading::processes::Resource::File(ref mut file_descriptor)) =
            crate::scheduler().current_process().resources.get_mut($ri)
        else {
            return Err(FSError::InvaildFileDescriptorOrRes);
        };

        file_descriptor
    }};
}

#[no_mangle]
pub fn open(path: Path) -> FSResult<usize> {
    let fd = vfs().open(path)?;
    Ok(scheduler().add_resource(Resource::File(fd)))
}

#[no_mangle]
pub fn close(ri: usize) -> FSResult<()> {
    let fd = get_fd!(ri);
    vfs().close(fd)?;

    _ = scheduler().remove_resource(ri);
    Ok(())
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
pub fn create(path: Path) -> FSResult<()> {
    vfs().create(path)
}

#[no_mangle]
pub fn createdir(path: Path) -> FSResult<()> {
    vfs().createdir(path)
}

pub const MAX_NAME_LEN: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct DirEntry {
    pub kind: InodeType,
    pub size: usize,
    pub name_length: usize,
    pub name: [u8; 128],
}

impl DirEntry {
    pub fn get_from_inode_with_name(inode: *const Inode, name: &str) -> FSResult<Self> {
        unsafe {
            let name_slice = name.as_bytes();
            let kind = (*inode).inode_type;
            let size = (*inode).size().unwrap_or(0);

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

    pub fn get_from_inode(inode: *const Inode) -> FSResult<Self> {
        unsafe { Self::get_from_inode_with_name(inode, &(*inode).name) }
    }

    pub const unsafe fn zeroed() -> Self {
        core::mem::zeroed()
    }
}

pub trait DirIter: Debug {
    fn next(&mut self) -> Option<&DirEntry>;
    fn clone(&self) -> Box<dyn DirIter>;
}

#[no_mangle]
/// opens a diriter as a resource
/// return the ri of the diriter
pub fn diriter_open(fd_ri: usize) -> FSResult<usize> {
    let fd = get_fd!(fd_ri);
    let diriter = vfs().diriter_open(fd)?;

    Ok(scheduler().add_resource(Resource::DirIter(diriter)))
}

pub fn diriter_next(dir_ri: usize, direntry: &mut DirEntry) -> FSResult<()> {
    let Some(Resource::DirIter(diriter)) = scheduler().current_process().resources.get_mut(dir_ri)
    else {
        return Err(FSError::InvaildFileDescriptorOrRes);
    };

    let next = diriter.next();
    if let Some(entry) = next {
        *direntry = entry.clone();
    } else {
        unsafe { *direntry = DirEntry::zeroed() }
    }
    Ok(())
}

#[no_mangle]
/// may only Err if dir_ri is invaild
pub fn diriter_close(dir_ri: usize) -> FSResult<()> {
    scheduler()
        .remove_resource(dir_ri)
        .ok()
        .ok_or(FSError::InvaildFileDescriptorOrRes)
}

#[no_mangle]
pub fn fstat(ri: usize, direntry: &mut DirEntry) -> FSResult<()> {
    let fd = get_fd!(ri);
    *direntry = DirEntry::get_from_inode(fd.node)?;
    Ok(())
}
