use alloc::{
    boxed::Box,
    string::{String, ToString},
    sync::Arc,
};
use spin::Mutex;

use crate::devices::{Device, DEVICE_MANAGER};

use super::{
    expose::{DirEntry, DirIter},
    FSResult, FileDescriptor, Inode, InodeOps, InodeType, Path, FS,
};

pub struct DeviceManagerInode;
impl InodeOps for Mutex<DeviceManagerInode> {
    fn inodeid(&self) -> usize {
        0
    }

    fn kind(&self) -> InodeType {
        InodeType::Directory
    }

    fn name(&self) -> String {
        "dev".to_string()
    }

    fn contains(&self, name: &str) -> bool {
        for device in DEVICE_MANAGER.lock().devices().iter() {
            if Device::name(*device) == name {
                return true;
            }
        }
        false
    }

    fn get(&self, name: &str) -> crate::drivers::vfs::FSResult<Option<usize>> {
        for (i, device) in DEVICE_MANAGER.lock().devices().iter().enumerate() {
            if Device::name(*device) == name {
                return Ok(Some(i + 1));
            }
        }
        Ok(None)
    }
}

#[derive(Clone)]
pub struct DeviceInode {
    inodeid: usize,
}

impl DeviceInode {
    pub fn new(inodeid: usize) -> Inode {
        Arc::new(Mutex::new(Self { inodeid }))
    }

    pub fn device(&self) -> &'static dyn Device {
        DEVICE_MANAGER
            .lock()
            .get_device_at(self.inodeid - 1)
            .unwrap()
    }
}

impl InodeOps for Mutex<DeviceInode> {
    fn name(&self) -> String {
        Device::name(self.lock().device()).to_string()
    }

    fn inodeid(&self) -> usize {
        self.lock().inodeid
    }

    fn kind(&self) -> InodeType {
        InodeType::Device
    }

    fn read(&self, buffer: &mut [u8], offset: usize, count: usize) -> FSResult<usize> {
        self.lock().device().read(buffer, offset, count)
    }

    fn write(&self, buffer: &[u8], offset: usize) -> FSResult<usize> {
        self.lock().device().write(buffer, offset)
    }
}

#[derive(Debug, Clone)]
pub struct DeviceDirIter {
    index: usize,
}
impl DeviceDirIter {
    pub fn new() -> Box<dyn DirIter> {
        Box::new(Self { index: 0 })
    }
}
impl DirIter for DeviceDirIter {
    fn next(&mut self) -> Option<DirEntry> {
        let index = self.index;
        self.index += 1;
        let name = Device::name(DEVICE_MANAGER.lock().get_device_at(index)?);
        let mut name_bytes = [0u8; 128];

        name_bytes[..name.len()].copy_from_slice(name.as_bytes());
        Some(DirEntry {
            kind: InodeType::Device,
            size: 0,
            name_length: name.len(),
            name: name_bytes,
        })
    }

    fn clone(&self) -> Box<dyn DirIter> {
        Box::new(Clone::clone(self))
    }
}
pub struct DeviceFS {
    root_inode: Inode,
}

impl DeviceFS {
    pub fn new() -> Self {
        Self {
            root_inode: Arc::new(Mutex::new(DeviceManagerInode)),
        }
    }
}

impl FS for DeviceFS {
    fn name(&self) -> &'static str {
        "devices"
    }

    fn root_inode(&self) -> FSResult<Inode> {
        Ok(self.root_inode.clone())
    }

    fn get_inode(&self, inode_id: usize) -> FSResult<Option<Inode>> {
        for (i, _) in DEVICE_MANAGER.lock().devices().iter().enumerate() {
            if i == inode_id - 1 {
                return Ok(Some(DeviceInode::new(inode_id)));
            }
        }

        Ok(None)
    }

    fn open(&mut self, path: Path) -> FSResult<FileDescriptor> {
        let resolved = self.reslove_path(path)?;
        Ok(FileDescriptor::new(self, resolved.clone()))
    }

    fn write(&mut self, file_descriptor: &mut FileDescriptor, buffer: &[u8]) -> FSResult<usize> {
        file_descriptor.node.write(buffer, 0)
    }

    fn read(&mut self, file_descriptor: &mut FileDescriptor, buffer: &mut [u8]) -> FSResult<usize> {
        file_descriptor.node.read(buffer, 0, 0)
    }

    fn diriter_open(&mut self, _fd: &mut FileDescriptor) -> FSResult<Box<dyn DirIter>> {
        Ok(DeviceDirIter::new())
    }
}
