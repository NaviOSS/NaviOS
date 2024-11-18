use core::usize;

use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec;
use alloc::{collections::btree_map::BTreeMap, string::String, vec::Vec};
use spin::Mutex;

use super::{DirIter, InodeOf};
use super::{FSError, FSResult, FileDescriptor, Inode, InodeOps, InodeType, Path, FS};

pub enum RamInodeData {
    Data(Vec<u8>),
    Children(BTreeMap<String, usize>),
    HardLink(Inode),
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

    fn new_hardlink(name: String, inode: Inode, inodeid: usize) -> InodeOf<Mutex<Self>> {
        Arc::new(RamInode::new(name, RamInodeData::HardLink(inode), inodeid))
    }
}

impl InodeOps for Mutex<RamInode> {
    fn size(&self) -> FSResult<usize> {
        match self.lock().data {
            RamInodeData::Data(ref data) => Ok(data.len()),
            _ => Err(FSError::NotAFile),
        }
    }
    fn get(&self, name: Path) -> FSResult<usize> {
        match self.lock().data {
            RamInodeData::Children(ref tree) => tree
                .get(name)
                .copied()
                .ok_or(FSError::NoSuchAFileOrDirectory),
            RamInodeData::HardLink(ref inode) => inode.get(name),
            _ => Err(FSError::NotADirectory),
        }
    }

    fn contains(&self, name: Path) -> bool {
        match self.lock().data {
            RamInodeData::Children(ref tree) => tree.contains_key(name),
            RamInodeData::HardLink(ref inode) => inode.contains(name),
            _ => false,
        }
    }

    fn truncate(&self, size: usize) -> FSResult<()> {
        match self.lock().data {
            RamInodeData::Data(ref mut data) => {
                data.truncate(size);
                Ok(())
            }
            RamInodeData::HardLink(ref inode) => inode.truncate(size),
            _ => Err(FSError::NotAFile),
        }
    }

    fn read(&self, buffer: &mut [u8], offset: usize, count: usize) -> FSResult<usize> {
        match self.lock().data {
            RamInodeData::Data(ref data) => {
                buffer[..count].copy_from_slice(&data[offset..offset + count]);
                Ok(count)
            }
            RamInodeData::HardLink(ref inode) => inode.read(buffer, offset, count),
            _ => Err(FSError::NotAFile),
        }
    }

    fn write(&self, buffer: &[u8], offset: usize) -> FSResult<usize> {
        match self.lock().data {
            RamInodeData::Data(ref mut data) => {
                if data.len() < buffer.len() + offset {
                    data.resize(buffer.len() + offset, 0);
                }

                data[offset..(offset + buffer.len())].copy_from_slice(buffer);
                Ok(buffer.len())
            }
            RamInodeData::HardLink(ref inode) => inode.write(buffer, offset),
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
            RamInodeData::HardLink(ref inode) => inode.insert(name, node),
            _ => Err(FSError::NotADirectory),
        }
    }

    fn kind(&self) -> InodeType {
        match self.lock().data {
            RamInodeData::Children(_) => InodeType::Directory,
            RamInodeData::Data(_) => InodeType::File,
            RamInodeData::HardLink(ref inode) => inode.kind(),
        }
    }

    fn name(&self) -> String {
        self.lock().name.clone()
    }

    fn inodeid(&self) -> usize {
        self.lock().inodeid
    }
    fn open_diriter(&self, fs: *mut dyn FS) -> FSResult<DirIter> {
        match self.lock().data {
            RamInodeData::Children(ref data) => Ok(DirIter::new(
                fs,
                data.into_iter().map(|(_, inodeid)| *inodeid).collect(),
            )),

            RamInodeData::HardLink(ref inode) => inode.open_diriter(fs),
            _ => Err(FSError::NotADirectory),
        }
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

    fn make_hardlink(&mut self, inodeid: usize, name: String) -> usize {
        let inode = self.inodes.get_mut(inodeid).unwrap();
        let inode = inode.clone();
        let inodeid = self.inodes.len();

        self.inodes
            .push(RamInode::new_hardlink(name, inode, inodeid));
        inodeid
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

    fn open(&self, path: Path) -> FSResult<FileDescriptor> {
        let file = self.reslove_path(path)?;
        let node = file.clone();

        Ok(FileDescriptor::new(
            self as *const RamFS as *mut RamFS,
            node,
        ))
    }

    fn read(&self, file_descriptor: &mut FileDescriptor, buffer: &mut [u8]) -> FSResult<usize> {
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

    fn write(&self, file_descriptor: &mut FileDescriptor, buffer: &[u8]) -> FSResult<usize> {
        if file_descriptor.write_pos == 0 {
            file_descriptor.node.truncate(0)?;
        }

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
        self.inodes.push(node.clone());

        let inodeid = self.make_hardlink(resloved.inodeid(), "..".to_string());
        node.insert("..", inodeid)?;

        Ok(())
    }
}
