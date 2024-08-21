use crate::utils::Locked;
pub mod ramfs;

use alloc::{
    boxed::Box,
    collections::btree_map::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};
use lazy_static::lazy_static;
use spin::MutexGuard;

lazy_static! {
    pub static ref VFS_STRUCT: Locked<VFS> = Locked::new(VFS::new());
}

pub fn vfs() -> MutexGuard<'static, VFS> {
    (*VFS_STRUCT).inner.lock()
}

pub fn vfs_init() {
    vfs().mount(b"ram", Box::new(ramfs::RamFS::new())).unwrap();
}

#[derive(Debug)]
#[repr(C)]
pub struct FileDescriptor {
    pub mountpoint: *mut dyn FS,
    pub node: *mut Inode,

    pub read_pos: usize,
    pub write_pos: usize,
}

impl FileDescriptor {
    pub fn size(&self) -> usize {
        unsafe { (*self.node).size() }
    }

    pub fn name(&self) -> String {
        unsafe { &*self.node }.name.clone()
    }
}

#[derive(Debug, Clone)]
pub enum FSError {
    OperationNotSupported,
    NotAFile,
    NotADirectory,
    NoSuchAFileOrDirectory,
    InvaildDrive,
}

pub type FSResult<T> = Result<T, FSError>;
#[derive(Clone, PartialEq)]
enum InodeType {
    File,
    Directory,
}

/// this Inode is pesudo too far this should only work with RamFS
/// i am trying to make it as generic as possible but i still dont have storage drivers and i have
/// no idea how these works am trying my best for now
pub struct Inode {
    name: String,
    inode_type: InodeType,
    ops: Box<dyn InodeOps>,
}
pub trait InodeOps: Send {
    fn new_root() -> Inode
    where
        Self: Sized;
    /// gets an Inode from self
    /// returns Err(()) if self is not a directory
    /// returns Ok(None) if self doesn't contain `name`
    fn get(&mut self, name: &String) -> FSResult<Option<&mut Inode>>;
    /// checks if node contains `name` returns false if it doesn't or if it is not a directory
    fn contains(&self, name: &String) -> bool;
    /// returns the size of node
    fn size(&self) -> usize;
    /// attempts to read `count` bytes of node data if it is a file
    /// panics if invaild `offset`
    fn read(&self, buffer: &mut [u8], offset: usize, count: usize) -> FSResult<()>;
    /// attempts to read the contents of self if it is a directory returning a list of inodes
    fn readdir(&mut self) -> FSResult<Vec<&mut Inode>>;
    /// attempts to write `buffer.len` bytes from `buffer` into node data if it is a file starting
    /// from offset
    /// extends the nodes data and node size if `buffer.len` + `offset` is greater then node size
    fn write(&mut self, buffer: &[u8], offset: usize) -> FSResult<()>;

    /// attempts to insert a node to self
    /// returns an FSError::NotADirectory if not a directory
    fn insert(&mut self, name: String, node: Inode) -> FSResult<()>;
}

impl Inode {
    /// quick wrapper around `self.ops.get`
    pub fn get(&mut self, name: &String) -> FSResult<Option<&mut Inode>> {
        self.ops.get(name)
    }

    /// quick wrapper around `self.ops.contains`
    fn contains(&self, name: &String) -> bool {
        self.ops.contains(name)
    }

    /// quick wrapper around `self.ops.size`
    fn size(&self) -> usize {
        self.ops.size()
    }

    pub fn is_dir(&self) -> bool {
        self.inode_type == InodeType::Directory
    }

    pub fn is_file(&self) -> bool {
        self.inode_type == InodeType::File
    }
}

pub trait FS: Send {
    /// returns the name of the fs
    /// for example, `TmpFS` name is "tmpfs"
    /// again we cannot use consts because of `dyn`...
    fn name(&self) -> &'static str;

    /// attempts to close a file cleanig all it's resources
    fn close(&mut self, file: FileDescriptor) -> FSResult<()>;

    fn root_inode_mut(&mut self) -> &mut Inode;

    /// goes trough path to get the inode it refers to
    fn reslove_path(&mut self, path: &String) -> Option<&mut Inode> {
        let path: Vec<&str> = path.split(&['/', '\\']).collect();

        let mut current_inode = self.root_inode_mut();
        let mut prev_inode = current_inode as *mut Inode;

        for depth in &path[1..] {
            if *depth == "" {
                break;
            }

            if *depth == "." {
                continue;
            }

            if *depth == ".." {
                current_inode = unsafe { &mut *prev_inode };
                continue;
            }

            if !current_inode.is_dir() {
                return None;
            }

            if !current_inode.contains(&depth.to_string()) {
                return None;
            }

            prev_inode = current_inode;
            current_inode = current_inode.get(&depth.to_string()).unwrap()?;
        }

        return Some(current_inode);
    }
    /// opens a path returning a file descriptor or an Err(()) if path doesn't exist
    fn open(&mut self, path: &String) -> FSResult<FileDescriptor>;
    /// attempts to read `buffer.len` bytes from file_descriptor returns the actual count of the bytes read
    fn read(&mut self, file_descriptor: &mut FileDescriptor, buffer: &mut [u8]) -> FSResult<usize>;
    /// attempts to read a directory returning it's content's FileDescriptors
    fn readdir(&mut self, file_descriptor: &mut FileDescriptor) -> FSResult<Vec<FileDescriptor>>;
    /// attempts to write `buffer.len` bytes to `file_descriptor`
    fn write(&mut self, file_descriptor: &mut FileDescriptor, buffer: &[u8]) -> FSResult<()>;
    /// creates an empty file named `name` in `path`
    fn create(&mut self, path: &String, name: String) -> FSResult<()>;
    /// creates an empty dir named `name` in `path`
    fn createdir(&mut self, path: &String, name: String) -> FSResult<()>;
}

pub struct VFS {
    pub drivers: BTreeMap<Vec<u8>, Box<dyn FS>>,
}

impl VFS {
    pub fn new() -> Self {
        Self {
            drivers: BTreeMap::new(),
        }
    }
    /// mounts a file system as a drive
    /// returns Err(()) if not enough memory or there is an already mounted driver with that
    /// name
    pub fn mount(&mut self, name: &[u8], value: Box<dyn FS>) -> Result<(), ()> {
        let name = name.to_vec();

        if self.drivers.contains_key(&name) {
            Err(())
        } else {
            self.drivers.insert(name, value);
            Ok(())
        }
    }

    /// unmounts a drive returns Err(()) if there is no such a drive
    pub fn umount(&mut self, name: &[u8]) -> Result<(), ()> {
        let name = name.to_vec();
        self.drivers.remove(&name).ok_or(())?;
        Ok(())
    }

    /// gets a drive from `self` named "`name`"
    /// or "`name`:" muttabily
    pub(self) fn get_with_name_mut(&mut self, name: &[u8]) -> Option<&mut Box<dyn FS>> {
        let mut name = name.to_vec();
        if name.ends_with(b":") {
            name.pop();
        }

        self.drivers.get_mut(&name)
    }

    /// gets a drive from `self` named "`name`"
    /// or "`name`:"
    pub fn get_with_name(&mut self, name: &[u8]) -> Option<&Box<dyn FS>> {
        let mut name = name.to_vec();
        if name.ends_with(b":") {
            name.pop();
        }

        self.drivers.get(&name)
    }

    /// gets the drive name from `path` then gets the drive
    /// path must be absolute starting with DRIVE_NAME:/
    pub(self) fn get_from_path(&mut self, path: &String) -> FSResult<Option<&mut Box<dyn FS>>> {
        let path: Vec<&str> = path.split(&['/', '\\']).collect();
        if path.len() <= 0 {
            return Err(FSError::InvaildDrive);
        }

        let drive = path[0];
        if !(drive.ends_with(':')) {
            return Err(FSError::InvaildDrive);
        }

        Ok(self.get_with_name_mut(drive.as_bytes()))
    }

    /// gets the path in mountpoint from a given path
    /// basiclly removes the drive name
    pub(self) fn mountpoint_path(path: &String) -> String {
        let index = path.find(':').unwrap();
        let path_in_mountpoint = &path[index + 1..path.len()];
        path_in_mountpoint.to_string()
    }

    /// checks if a path is a vaild dir returns Err if path has an error
    pub fn verify_path_dir(&mut self, path: &String) -> FSResult<()> {
        let mountpoint = self
            .get_from_path(path)?
            .ok_or(FSError::NoSuchAFileOrDirectory)?;
        let res = mountpoint
            .reslove_path(path)
            .ok_or(FSError::NoSuchAFileOrDirectory)?;

        if !res.is_dir() {
            return Err(FSError::NotADirectory);
        }
        Ok(())
    }
}

impl FS for VFS {
    fn name(&self) -> &'static str {
        "vfs"
    }

    fn root_inode_mut(&mut self) -> &mut Inode {
        unreachable!()
    }

    fn reslove_path(&mut self, _path: &String) -> Option<&mut Inode> {
        unreachable!()
    }

    fn open(&mut self, path: &String) -> FSResult<FileDescriptor> {
        let mountpoint = self
            .get_from_path(path)?
            .ok_or(FSError::NoSuchAFileOrDirectory)?;

        let file = mountpoint.open(&Self::mountpoint_path(path))?;

        Ok(file)
    }

    fn read(&mut self, file_descriptor: &mut FileDescriptor, buffer: &mut [u8]) -> FSResult<usize> {
        unsafe { (*file_descriptor.mountpoint).read(file_descriptor, buffer) }
    }

    fn readdir(&mut self, file_descriptor: &mut FileDescriptor) -> FSResult<Vec<FileDescriptor>> {
        unsafe { (*file_descriptor.mountpoint).readdir(file_descriptor) }
    }

    fn write(&mut self, file_descriptor: &mut FileDescriptor, buffer: &[u8]) -> FSResult<()> {
        unsafe { (*file_descriptor.mountpoint).write(file_descriptor, buffer) }
    }

    fn create(&mut self, path: &String, name: String) -> FSResult<()> {
        let mountpoint = self
            .get_from_path(path)?
            .ok_or(FSError::NoSuchAFileOrDirectory)?;

        mountpoint.create(path, name)
    }

    fn createdir(&mut self, path: &String, name: String) -> FSResult<()> {
        let mountpoint = self
            .get_from_path(path)?
            .ok_or(FSError::NoSuchAFileOrDirectory)?;

        mountpoint.createdir(path, name)
    }

    fn close(&mut self, file: FileDescriptor) -> FSResult<()> {
        unsafe { (*file.mountpoint).close(file) }
    }
}
