//! exposed functions of VFS they manually uses
//! a resource index instead of a file descriptor aka ri
use core::{fmt::Debug, str, usize};

use crate::threading::{
    expose::{add_resource, get_resource, remove_resource},
    resources::Resource,
};

use super::{FSError, FSResult, Inode, InodeType, Path, FS, VFS_STRUCT};
/// gets a FileDescriptor from a fd (file_descriptor id) may return Err(FSError::InvaildFileDescriptor)
macro_rules! get_fd {
    ($ri: expr) => {{
        let Some(crate::threading::resources::Resource::File(ref mut file_descriptor)) =
            crate::threading::expose::get_resource($ri)
        else {
            return Err(FSError::NotAFile);
        };

        file_descriptor
    }};
}

#[no_mangle]
pub fn open(path: Path) -> FSResult<usize> {
    let fd = VFS_STRUCT
        .try_read()
        .ok_or(FSError::ResourceBusy)?
        .open(path)?;
    Ok(add_resource(Resource::File(fd)))
}

#[no_mangle]
pub fn close(ri: usize) -> FSResult<()> {
    let fd = get_fd!(ri);
    VFS_STRUCT
        .try_read()
        .ok_or(FSError::ResourceBusy)?
        .close(fd)?;

    _ = remove_resource(ri);
    Ok(())
}

#[no_mangle]
pub fn read(ri: usize, buffer: &mut [u8]) -> FSResult<usize> {
    let fd = get_fd!(ri);
    VFS_STRUCT
        .try_read()
        .ok_or(FSError::ResourceBusy)?
        .read(fd, buffer)
}

#[no_mangle]
pub fn write(ri: usize, buffer: &[u8]) -> FSResult<usize> {
    let fd = get_fd!(ri);
    VFS_STRUCT
        .try_read()
        .ok_or(FSError::ResourceBusy)?
        .write(fd, buffer)
}

#[no_mangle]
pub fn create(path: Path) -> FSResult<()> {
    VFS_STRUCT
        .try_write()
        .ok_or(FSError::ResourceBusy)?
        .create(path)
}

#[no_mangle]
pub fn createdir(path: Path) -> FSResult<()> {
    VFS_STRUCT
        .try_write()
        .ok_or(FSError::ResourceBusy)?
        .createdir(path)
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
    #[inline]
    pub fn name(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.name[..self.name_length]) }
    }

    pub fn get_from_inode(inode: Inode) -> FSResult<Self> {
        let name = inode.name();
        let name_slice = name.as_bytes();

        let kind = inode.kind();
        let size = inode.size().unwrap_or(0);

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

    pub const unsafe fn zeroed() -> Self {
        core::mem::zeroed()
    }
}

#[no_mangle]
/// opens a diriter as a resource
/// return the ri of the diriter
pub fn diriter_open(fd_ri: usize) -> FSResult<usize> {
    let fd = get_fd!(fd_ri);
    let diriter = VFS_STRUCT
        .try_read()
        .ok_or(FSError::ResourceBusy)?
        .diriter_open(fd)?;

    Ok(add_resource(Resource::DirIter(diriter)))
}

pub fn diriter_next(dir_ri: usize, direntry: &mut DirEntry) -> FSResult<()> {
    let Some(Resource::DirIter(diriter)) = get_resource(dir_ri) else {
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
    remove_resource(dir_ri).map_err(|_| FSError::InvaildFileDescriptorOrRes)
}

#[no_mangle]
pub fn fstat(ri: usize, direntry: &mut DirEntry) -> FSResult<()> {
    let fd = get_fd!(ri);
    *direntry = DirEntry::get_from_inode(fd.node.clone())?;
    Ok(())
}
