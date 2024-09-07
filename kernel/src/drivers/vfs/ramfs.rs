use core::usize;

use alloc::{boxed::Box, collections::btree_map::BTreeMap, string::String, sync::Arc, vec::Vec};
use spin::Mutex;

use super::{
    expose::{DirEntry, DirIter},
    FSError, FSResult, FileDescriptor, Inode, InodeOps, InodeType, Path, FS,
};

pub enum RamInode {
    Data(Vec<u8>),
    Children(BTreeMap<String, Inode>),
    Symlink(Arc<Mutex<dyn InodeOps>>),
}

impl RamInode {
    fn new_file(name: String, data: &[u8]) -> Inode {
        Inode {
            name,
            inode_type: InodeType::File,
            ops: Box::new(RamInode::Data(data.to_vec())),
        }
    }

    fn new_dir(name: String) -> Inode {
        Inode {
            name,
            inode_type: InodeType::Directory,
            ops: Box::new(RamInode::Children(BTreeMap::new())),
        }
    }
}

impl InodeOps for RamInode {
    fn new_root() -> Inode {
        Inode {
            name: String::new(),
            inode_type: InodeType::Directory,
            ops: Box::new(RamInode::Children(BTreeMap::new())),
        }
    }

    fn get(&mut self, name: Path) -> FSResult<Option<&mut Inode>> {
        match self {
            Self::Children(tree) => Ok(tree.get_mut(name)),
            _ => Err(FSError::NotADirectory),
        }
    }

    fn contains(&self, name: Path) -> bool {
        match self {
            Self::Children(tree) => tree.contains_key(name),
            _ => false,
        }
    }

    fn size(&self) -> usize {
        match self {
            Self::Data(data) => data.len(),
            Self::Children(children) => {
                let mut total = 0;
                for child in children.values() {
                    total += child.ops.size()
                }

                total
            }
            Self::Symlink(_) => todo!(),
        }
    }

    fn read(&self, buffer: &mut [u8], offset: usize, count: usize) -> FSResult<()> {
        match self {
            Self::Data(data) => Ok(buffer[..count].copy_from_slice(&data[offset..offset + count])),
            _ => Err(FSError::NotAFile),
        }
    }

    fn readdir(&mut self) -> FSResult<Vec<&mut Inode>> {
        match self {
            Self::Children(tree) => Ok(tree.values_mut().collect()),
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

    fn insert(&mut self, name: String, node: Inode) -> FSResult<()> {
        match self {
            Self::Children(tree) => {
                tree.insert(name, node);
                Ok(())
            }
            _ => Err(FSError::NotADirectory),
        }
    }
}

pub struct RamDirIter<'a> {
    entries: Vec<&'a mut Inode>,
    index: usize,
}

impl<'a> Iterator for RamDirIter<'a> {
    type Item = DirEntry;
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        let entry = self.entries.get(index)?;
        self.index += 1;
        Some(DirEntry::get_from_inode(*entry).ok()?)
    }
}

impl<'a> DirIter for RamDirIter<'a> {}

pub struct RamFS {
    root_inode: Inode,
}

impl RamFS {
    pub fn new() -> Self {
        Self {
            root_inode: RamInode::new_root(),
        }
    }
}

impl FS for RamFS {
    fn name(&self) -> &'static str {
        "ramfs"
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
            if file_descriptor.read_pos + count > (*file_descriptor.node).size() {
                count - (file_descriptor.read_pos + count - (*file_descriptor.node).size())
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
        let node = RamInode::new_file(name, &[]);

        let resloved = self.reslove_path(path)?;
        if resloved.inode_type != InodeType::Directory {
            return Err(FSError::NotADirectory);
        }

        resloved.ops.insert(node.name.clone(), node)?;
        Ok(())
    }

    fn createdir(&mut self, path: Path, name: String) -> FSResult<()> {
        let node = RamInode::new_dir(name);

        let resloved = self.reslove_path(path)?;
        if resloved.inode_type != InodeType::Directory {
            return Err(FSError::NotADirectory);
        }

        resloved.ops.insert(node.name.clone(), node)?;
        Ok(())
    }

    fn diriter_open(&mut self, fd: &mut FileDescriptor) -> FSResult<Box<dyn DirIter>> {
        let entries = unsafe { (*fd.node).ops.readdir()? };

        Ok(Box::new(RamDirIter { entries, index: 0 }))
    }

    fn root_inode_mut(&mut self) -> &mut Inode {
        &mut self.root_inode
    }
}
