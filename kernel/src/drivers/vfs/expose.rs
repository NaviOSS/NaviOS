use core::usize;

use alloc::{string::String, vec::Vec};

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
    pub read_pos: usize,
    pub write_pos: usize,
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
            read_pos: file_descriptor.read_pos,
            write_pos: file_descriptor.write_pos,
        };

        Ok(())
    }

    pub fn get_from_inode(inode: *mut Inode) -> FSResult<Self> {
        unsafe {
            let kind = (*inode).inode_type;
            let size = (*inode).size();
            Ok(Self {
                inode,
                kind,
                size,
                read_pos: 0,
                write_pos: 0,
            })
        }
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
pub fn create(path: Path, name: String) -> FSResult<()> {
    vfs().create(path, name)
}

#[no_mangle]
pub fn createdir(path: Path, name: String) -> FSResult<()> {
    vfs().createdir(path, name)
}

#[no_mangle]
/// reads a directory appending all of it's FileDescriptorStats to buffer
pub fn readdir(ri: usize, buffer: &mut [FileDescriptorStat]) -> FSResult<()> {
    let mut stats = unsafe { FileDescriptorStat::default() };
    FileDescriptorStat::get(ri, &mut stats)?;

    if stats.kind != InodeType::Directory {
        return Err(FSError::NotADirectory);
    }

    let entries = unsafe { (*(stats.inode)).ops.readdir()? };
    if entries.len() != buffer.len() {
        return Err(FSError::InvaildBuffer);
    }

    let mut entries_stats = Vec::with_capacity(entries.len());

    for entry in entries {
        entries_stats.push(FileDescriptorStat::get_from_inode(entry)?);
    }

    buffer.copy_from_slice(&entries_stats);
    Ok(())
}

#[no_mangle]
pub fn direntrycount(ri: usize) -> FSResult<usize> {
    let fd = get_fd!(ri);
    unsafe { Ok((*fd.node).ops.readdir()?.len()) }
}
