use core::{fmt::Debug, usize};

use alloc::string::ToString;
use alloc::vec;
use alloc::{boxed::Box, collections::btree_map::BTreeMap, string::String, vec::Vec};

use super::{
    expose::{DirEntry, DirIter},
    FSError, FSResult, FileDescriptor, Inode, InodeOps, InodeType, Path, FS,
};

pub enum RamInode {
    Data(Vec<u8>),
    Children(BTreeMap<String, usize>),
}

impl RamInode {
    fn new_file(name: String, data: &[u8], inodeid: usize) -> Inode {
        Inode {
            name,
            inode_type: InodeType::File,
            inodeid,
            ops: Box::new(RamInode::Data(data.to_vec())),
        }
    }

    fn new_dir(name: String, inodeid: usize) -> Inode {
        Inode {
            name,
            inode_type: InodeType::Directory,
            inodeid,
            ops: Box::new(RamInode::Children(BTreeMap::new())),
        }
    }
}

impl InodeOps for RamInode {
    fn size(&self) -> FSResult<usize> {
        match self {
            Self::Data(data) => Ok(data.len()),
            Self::Children(_) => Err(FSError::NotAFile),
        }
    }
    fn get(&self, name: Path) -> FSResult<Option<usize>> {
        match self {
            Self::Children(tree) => Ok(tree.get(name).copied()),
            _ => Err(FSError::NotADirectory),
        }
    }

    fn contains(&self, name: Path) -> bool {
        match self {
            Self::Children(tree) => tree.contains_key(name),
            _ => false,
        }
    }

    fn read(&self, buffer: &mut [u8], offset: usize, count: usize) -> FSResult<()> {
        match self {
            Self::Data(data) => Ok(buffer[..count].copy_from_slice(&data[offset..offset + count])),
            _ => Err(FSError::NotAFile),
        }
    }

    fn readdir(&mut self) -> FSResult<Vec<usize>> {
        match self {
            Self::Children(tree) => Ok(tree.values().copied().collect()),
            _ => Err(FSError::NotADirectory),
        }
    }

    fn write(&mut self, buffer: &[u8], offset: usize) -> FSResult<()> {
        match self {
            Self::Data(data) => {
                if data.len() < buffer.len() + offset {
                    data.resize(buffer.len() + offset, 0);
                }

                data[offset..buffer.len()].copy_from_slice(buffer);
                Ok(())
            }
            _ => Err(FSError::NotAFile),
        }
    }

    fn insert(&mut self, name: String, node: usize) -> FSResult<()> {
        match self {
            Self::Children(tree) => {
                tree.insert(name, node);
                Ok(())
            }
            _ => Err(FSError::NotADirectory),
        }
    }
}

pub struct RamDirIter {
    fs: *const dyn FS,
    entries: Vec<usize>,
    pub index: usize,
}
impl Debug for RamDirIter {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "RamDirIter {{ index: {} }}", self.index)
    }
}

impl Iterator for RamDirIter {
    type Item = DirEntry;
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        let entry = *self.entries.get(index)?;
        self.index += 1;

        let node = unsafe { (*self.fs).get_inode(entry).unwrap()? };
        Some(DirEntry::get_from_inode(node).ok()?)
    }
}

impl DirIter for RamDirIter {}

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
    fn get_inode(&self, inode_id: usize) -> FSResult<Option<&Inode>> {
        Ok(self.inodes.get(inode_id))
    }
    #[inline]
    fn get_inode_mut(&mut self, inode_id: usize) -> FSResult<Option<&mut Inode>> {
        Ok(self.inodes.get_mut(inode_id))
    }

    fn open(&mut self, path: Path) -> FSResult<FileDescriptor> {
        let file = self.reslove_path(path)?;

        let file = file as *mut Inode;
        Ok(FileDescriptor {
            mountpoint: self,

            write_pos: 0,
            read_pos: 0,
            node: file,
        })
    }

    fn read(&mut self, file_descriptor: &mut FileDescriptor, buffer: &mut [u8]) -> FSResult<usize> {
        let count = buffer.len();
        let count = unsafe {
            if file_descriptor.read_pos + count > (*file_descriptor.node).size()? {
                count - (file_descriptor.read_pos + count - (*file_descriptor.node).size()?)
            } else {
                count
            }
        };

        unsafe {
            (*file_descriptor.node)
                .ops
                .read(buffer, file_descriptor.read_pos, count)?;
        }

        file_descriptor.read_pos += count;
        Ok(count)
    }

    fn write(&mut self, file_descriptor: &mut FileDescriptor, buffer: &[u8]) -> FSResult<()> {
        unsafe {
            (*file_descriptor.node)
                .ops
                .write(buffer, file_descriptor.write_pos)?;
        }

        file_descriptor.write_pos += buffer.len();

        Ok(())
    }

    fn create(&mut self, path: Path, name: String) -> FSResult<()> {
        let inodeid = self.inodes.len();
        let name_clone = name.clone();
        let node = RamInode::new_file(name, &[], inodeid);

        self.inodes.push(node);

        let resloved = self.reslove_path(path)?;

        if resloved.inode_type != InodeType::Directory {
            return Err(FSError::NotADirectory);
        }

        resloved.ops.insert(name_clone, inodeid)?;
        Ok(())
    }

    fn createdir(&mut self, path: Path, name: String) -> FSResult<()> {
        let inodeid = self.inodes.len();
        let name_clone = name.clone();
        let node = RamInode::new_dir(name, inodeid);
        self.inodes.push(node);

        let resloved = self.reslove_path(path)?;
        if resloved.inode_type != InodeType::Directory {
            return Err(FSError::NotADirectory);
        }

        resloved.ops.insert(name_clone, inodeid)?;
        Ok(())
    }

    fn diriter_open(&mut self, fd: &mut FileDescriptor) -> FSResult<Box<dyn DirIter>> {
        let entries = unsafe { (*fd.node).ops.readdir()? };

        Ok(Box::new(RamDirIter {
            fs: self,
            entries,
            index: 0,
        }))
    }
}
