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

    fn insert(&mut self, name: &str, node: usize) -> FSResult<()> {
        match self {
            Self::Children(tree) => {
                if tree.contains_key(name) {
                    return Err(FSError::AlreadyExists);
                }

                tree.insert(name.to_string(), node);
                Ok(())
            }
            _ => Err(FSError::NotADirectory),
        }
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
    fn next(&mut self) -> Option<&DirEntry> {
        let index = self.index;
        self.index += 1;

        self.dir.get(index)
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
        let file_size = unsafe { (*file_descriptor.node).size()? };

        let count = if file_descriptor.read_pos + count > file_size {
            file_size - file_descriptor.read_pos
        } else {
            count
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

    fn create(&mut self, path: Path) -> FSResult<()> {
        let inodeid = self.inodes.len();

        let (resloved, name) = self.reslove_path_uncreated(path)?;
        resloved.ops.insert(name, inodeid)?;

        let node = RamInode::new_file(name.to_string(), &[], inodeid);
        self.inodes.push(node);

        Ok(())
    }

    fn createdir(&mut self, path: Path) -> FSResult<()> {
        let inodeid = self.inodes.len();

        let (resloved, name) = self.reslove_path_uncreated(path)?;
        resloved.ops.insert(name, inodeid)?;

        let mut node = RamInode::new_dir(name.to_string(), inodeid);
        node.ops.insert("..", resloved.inodeid)?;

        self.inodes.push(node);

        Ok(())
    }

    fn diriter_open(&mut self, fd: &mut FileDescriptor) -> FSResult<Box<dyn DirIter>> {
        if unsafe { !(*fd.node).is_dir() } {
            return Err(FSError::NotADirectory);
        }

        let raminode: *const RamInode =
            unsafe { ((*fd.node).ops.as_ref() as *const dyn InodeOps).cast() };

        let data = match unsafe { &*raminode } {
            RamInode::Children(ref data) => data,
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
