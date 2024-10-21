use core::{fmt::Debug, usize};

use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec;
use alloc::{boxed::Box, collections::btree_map::BTreeMap, string::String, vec::Vec};
use spin::Mutex;

use super::InodeOf;
use super::{
    expose::{DirEntry, DirIter},
    FSError, FSResult, FileDescriptor, Inode, InodeOps, InodeType, Path, FS,
};

pub enum RamInodeData {
    Data(Vec<u8>),
    Children(BTreeMap<String, usize>),
}

pub struct RamInode {
    name: String,
    data: RamInodeData,
    inodeid: usize,
}
impl RamInode {
    fn new(name: String, data: RamInodeData, inodeid: usize) -> Mutex<Self> {
        Mutex::new(Self {
            name,
            data,
            inodeid,
        })
    }

    fn new_file(name: String, data: &[u8], inodeid: usize) -> InodeOf<Mutex<Self>> {
        Arc::new(RamInode::new(
            name,
            RamInodeData::Data(data.to_vec()),
            inodeid,
        ))
    }

    fn new_dir(name: String, inodeid: usize) -> InodeOf<Mutex<Self>> {
        Arc::new(RamInode::new(
            name,
            RamInodeData::Children(BTreeMap::new()),
            inodeid,
        ))
    }
}

impl InodeOps for Mutex<RamInode> {
    fn size(&self) -> FSResult<usize> {
        match self.lock().data {
            RamInodeData::Data(ref data) => Ok(data.len()),
            RamInodeData::Children(_) => Err(FSError::NotAFile),
        }
    }
    fn get(&self, name: Path) -> FSResult<Option<usize>> {
        match self.lock().data {
            RamInodeData::Children(ref tree) => Ok(tree.get(name).copied()),
            _ => Err(FSError::NotADirectory),
        }
    }

    fn contains(&self, name: Path) -> bool {
        match self.lock().data {
            RamInodeData::Children(ref tree) => tree.contains_key(name),
            _ => false,
        }
    }

    fn read(&self, buffer: &mut [u8], offset: usize, count: usize) -> FSResult<usize> {
        match self.lock().data {
            RamInodeData::Data(ref data) => {
                buffer[..count].copy_from_slice(&data[offset..offset + count]);
                Ok(count)
            }
            _ => Err(FSError::NotAFile),
        }
    }

    fn write(&self, buffer: &[u8], offset: usize) -> FSResult<usize> {
        match self.lock().data {
            RamInodeData::Data(ref mut data) => {
                if data.len() < buffer.len() + offset {
                    data.resize(buffer.len() + offset, 0);
                }

                data[offset..buffer.len()].copy_from_slice(buffer);
                Ok(buffer.len() - offset)
            }
            _ => Err(FSError::NotAFile),
        }
    }

    fn insert(&self, name: &str, node: usize) -> FSResult<()> {
        match self.lock().data {
            RamInodeData::Children(ref mut tree) => {
                if tree.contains_key(name) {
                    return Err(FSError::AlreadyExists);
                }

                tree.insert(name.to_string(), node);
                Ok(())
            }
            _ => Err(FSError::NotADirectory),
        }
    }

    fn kind(&self) -> InodeType {
        match self.lock().data {
            RamInodeData::Children(_) => InodeType::Directory,
            RamInodeData::Data(_) => InodeType::File,
        }
    }

    fn name(&self) -> String {
        self.lock().name.clone()
    }

    fn inodeid(&self) -> usize {
        self.lock().inodeid
    }
}

#[derive(Clone)]
pub struct RamDirIter {
    index: usize,
    dir: Vec<DirEntry>,
}

impl Debug for RamDirIter {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "RamDirIter")
    }
}

impl DirIter for RamDirIter {
    fn next(&mut self) -> Option<DirEntry> {
        let index = self.index;
        self.index += 1;

        self.dir.get(index).cloned()
    }

    fn clone(&self) -> Box<dyn DirIter> {
        Box::new(Clone::clone(self))
    }
}

pub struct RamFS {
    inodes: Vec<Inode>,
}

impl RamFS {
    pub fn new() -> Self {
        Self {
            inodes: vec![RamInode::new_dir("/".to_string(), 0)],
        }
    }
}

impl FS for RamFS {
    fn name(&self) -> &'static str {
        "ramfs"
    }

    #[inline]
    fn get_inode(&self, inode_id: usize) -> FSResult<Option<Inode>> {
        let node = self.inodes.get(inode_id);
        Ok(node.map(|node| node.clone()))
    }

    fn open(&mut self, path: Path) -> FSResult<FileDescriptor> {
        let file = self.reslove_path(path)?;
        let node = file.clone();

        Ok(FileDescriptor::new(self, node))
    }

    fn read(&mut self, file_descriptor: &mut FileDescriptor, buffer: &mut [u8]) -> FSResult<usize> {
        let count = buffer.len();
        let file_size = file_descriptor.node.size()?;

        let count = if file_descriptor.read_pos + count > file_size {
            file_size - file_descriptor.read_pos
        } else {
            count
        };

        file_descriptor
            .node
            .read(buffer, file_descriptor.read_pos, count)?;

        file_descriptor.read_pos += count;
        Ok(count)
    }

    fn write(&mut self, file_descriptor: &mut FileDescriptor, buffer: &[u8]) -> FSResult<usize> {
        file_descriptor
            .node
            .write(buffer, file_descriptor.write_pos)?;

        file_descriptor.write_pos += buffer.len();

        Ok(buffer.len())
    }

    fn create(&mut self, path: Path) -> FSResult<()> {
        let inodeid = self.inodes.len();

        let (resloved, name) = self.reslove_path_uncreated(path)?;
        resloved.insert(name, inodeid)?;

        let node = RamInode::new_file(name.to_string(), &[], inodeid);
        self.inodes.push(node);

        Ok(())
    }

    fn createdir(&mut self, path: Path) -> FSResult<()> {
        let inodeid = self.inodes.len();

        let (resloved, name) = self.reslove_path_uncreated(path)?;
        resloved.insert(name, inodeid)?;

        let node = RamInode::new_dir(name.to_string(), inodeid);
        node.insert("..", resloved.inodeid())?;

        self.inodes.push(node);

        Ok(())
    }

    fn diriter_open(&mut self, fd: &mut FileDescriptor) -> FSResult<Box<dyn DirIter>> {
        // TODO: safer way to do this
        // by adding an InodeOp that returns a Box<dyn DirIter> from self
        if !fd.node.is_dir() {
            return Err(FSError::NotADirectory);
        }

        let raminode: *const Mutex<RamInode> = (&raw const *fd.node).cast();
        let lock = unsafe { &*raminode }.lock();

        let data = match lock.data {
            RamInodeData::Children(ref data) => data,
            _ => unreachable!(),
        };
        let mut data_entries = Vec::with_capacity(data.len());

        for (name, inode_id) in data {
            let inode = self.get_inode(*inode_id)?.unwrap();

            data_entries.push(DirEntry::get_from_inode_with_name(inode, &name)?)
        }

        Ok(Box::new(RamDirIter {
            dir: data_entries,
            index: 0,
        }))
    }
}
