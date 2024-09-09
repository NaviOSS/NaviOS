pub mod expose;

use core::usize;

use crate::{debug, utils::Locked};
pub mod ramfs;

use alloc::{
    boxed::Box,
    collections::btree_map::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};
use expose::DirIter;
use lazy_static::lazy_static;
use spin::MutexGuard;
pub type Path<'a> = &'a str;

lazy_static! {
    pub static ref VFS_STRUCT: Locked<VFS> = Locked::new(VFS::new());
}

pub fn vfs() -> MutexGuard<'static, VFS> {
    (*VFS_STRUCT).inner.lock()
}

pub fn init() {
    debug!(VFS, "initing ...");
    let mut vfs = vfs();
    let ramfs = Box::new(ramfs::RamFS::new());
    vfs.mount(b"ram", ramfs).unwrap();
    debug!(VFS, "done ...");
}

#[derive(Debug)]
pub struct FileDescriptor {
    pub mountpoint: *mut dyn FS,
    pub node: *mut Inode,
    /// acts as a dir entry index for directories
    /// acts as a byte index for files
    pub read_pos: usize,
    /// acts as a byte index for files
    /// doesn't do anything for directories
    pub write_pos: usize,
}

impl FileDescriptor {
    pub fn size(&self) -> usize {
        unsafe { (*self.node).size() }
    }

    pub fn name(&self) -> String {
        unsafe { (*self.node).name.clone() }
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum FSError {
    OperationNotSupported,
    NotAFile,
    NotADirectory,
    NoSuchAFileOrDirectory,
    InvaildDrive,
    InvaildPath,
    /// ethier a fd which points to a resource which isnt a FileDescriptor or it points to nothing
    InvaildFileDescriptorOrRes,
    InvaildBuffer,
}

pub type FSResult<T> = Result<T, FSError>;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InodeType {
    File,
    Directory,
    Symlink,
}

/// this Inode is pesudo too far this should only work with RamFS
/// i am trying to make it as generic as possible but i still dont have storage drivers and i have
/// no idea how these works am trying my best for now
/// TODO: system checklist
/// - Synchronization
/// - Allocations
/// TODO: inode id!

pub struct Inode {
    name: String,
    inode_type: InodeType,
    ///  TODO: use something instead of Box
    ops: Box<dyn InodeOps>,
}

pub trait InodeOps: Send {
    fn new_root() -> Inode
    where
        Self: Sized;
    /// gets an Inode from self
    /// returns Err(()) if self is not a directory
    /// returns Ok(None) if self doesn't contain `name`
    fn get(&mut self, name: &str) -> FSResult<Option<&mut Inode>>;
    /// checks if node contains `name` returns false if it doesn't or if it is not a directory
    fn contains(&self, name: &str) -> bool;
    /// returns the size of node
    fn size(&self) -> usize;
    /// attempts to read `count` bytes of node data if it is a file
    /// panics if invaild `offset`
    fn read(&self, buffer: &mut [u8], offset: usize, count: usize) -> FSResult<()> {
        _ = buffer;
        _ = offset;
        _ = count;
        Err(FSError::OperationNotSupported)
    }
    /// attempts to read the contents of self if it is a directory returning a list of inodes
    fn readdir(&mut self) -> FSResult<Vec<&mut Inode>> {
        Err(FSError::OperationNotSupported)
    }
    /// attempts to write `buffer.len` bytes from `buffer` into node data if it is a file starting
    /// from offset
    /// extends the nodes data and node size if `buffer.len` + `offset` is greater then node size
    fn write(&mut self, buffer: &[u8], offset: usize) -> FSResult<()> {
        _ = buffer;
        _ = offset;
        Err(FSError::OperationNotSupported)
    }

    /// attempts to insert a node to self
    /// returns an FSError::NotADirectory if not a directory
    fn insert(&mut self, name: String, node: Inode) -> FSResult<()> {
        _ = name;
        _ = node;
        Err(FSError::OperationNotSupported)
    }
}

impl Inode {
    /// quick wrapper around `self.ops.get`
    pub fn get(&mut self, name: &str) -> FSResult<Option<&mut Inode>> {
        self.ops.get(name)
    }

    /// quick wrapper around `self.ops.contains`
    fn contains(&self, name: &str) -> bool {
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
    fn close(&mut self, file_descriptor: &mut FileDescriptor) -> FSResult<()> {
        _ = file_descriptor;
        Ok(())
    }

    fn root_inode_mut(&mut self) -> &mut Inode;

    /// goes trough path to get the inode it refers to
    /// will err if there is no such a file or directory or path is straight up invaild
    fn reslove_path(&mut self, path: Path) -> FSResult<&mut Inode> {
        let mut path = path.split(&['/', '\\']);

        let mut current_inode = self.root_inode_mut();
        let mut prev_inode = current_inode as *mut Inode;
        path.next();

        while let Some(depth) = path.next() {
            if depth == "" {
                if path.next() == None {
                    break;
                } else {
                    return Err(FSError::InvaildPath);
                }
            }

            if depth == "." {
                continue;
            }

            if depth == ".." {
                current_inode = unsafe { &mut *prev_inode };
                continue;
            }

            if !current_inode.is_dir() {
                return Err(FSError::NoSuchAFileOrDirectory);
            }

            if !current_inode.contains(depth) {
                return Err(FSError::NoSuchAFileOrDirectory);
            }

            prev_inode = current_inode;
            current_inode = current_inode.get(depth)?.unwrap();
        }

        return Ok(current_inode);
    }
    /// opens a path returning a file descriptor or an Err(()) if path doesn't exist
    fn open(&mut self, path: Path) -> FSResult<FileDescriptor>;
    /// attempts to read `buffer.len` bytes from file_descriptor returns the actual count of the bytes read
    /// shouldn't read directories!
    fn read(&mut self, file_descriptor: &mut FileDescriptor, buffer: &mut [u8]) -> FSResult<usize> {
        _ = file_descriptor;
        _ = buffer;
        Err(FSError::OperationNotSupported)
    }
    /// attempts to write `buffer.len` bytes to `file_descriptor`
    /// shouldn't write to directories!
    fn write(&mut self, file_descriptor: &mut FileDescriptor, buffer: &[u8]) -> FSResult<()> {
        _ = file_descriptor;
        _ = buffer;
        Err(FSError::OperationNotSupported)
    }
    /// creates an empty file named `name` in `path`
    fn create(&mut self, path: Path, name: String) -> FSResult<()> {
        _ = path;
        _ = name;
        Err(FSError::OperationNotSupported)
    }
    /// creates an empty dir named `name` in `path`
    fn createdir(&mut self, path: Path, name: String) -> FSResult<()> {
        _ = path;
        _ = name;
        Err(FSError::OperationNotSupported)
    }

    /// opens an iterator of directroy entires, fd must be a directory
    fn diriter_open(&mut self, fd: &mut FileDescriptor) -> FSResult<Box<dyn DirIter>>;
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
        self.drivers.remove(name).ok_or(())?;
        Ok(())
    }

    /// gets a drive from `self` named "`name`"
    /// or "`name`:" muttabily
    pub(self) fn get_with_name_mut(&mut self, name: &[u8]) -> Option<&mut Box<dyn FS>> {
        let mut name = name;

        if name.ends_with(b":") {
            name = &name[..name.len() - 1];
        }

        self.drivers.get_mut(name)
    }

    /// gets a drive from `self` named "`name`"
    /// or "`name`:"
    pub fn get_with_name(&mut self, name: &[u8]) -> Option<&Box<dyn FS>> {
        let mut name = name;

        if name.ends_with(b":") {
            name = &name[..name.len() - 1];
        }

        self.drivers.get(name)
    }

    /// gets the drive name from `path` then gets the drive
    /// path must be absolute starting with DRIVE_NAME:/
    pub(self) fn get_from_path(&mut self, path: Path) -> FSResult<Option<&mut Box<dyn FS>>> {
        let mut path = path.split(&['/', '\\']);

        let drive = path.next().ok_or(FSError::InvaildDrive)?;
        if !(drive.ends_with(':')) {
            return Err(FSError::InvaildDrive);
        }

        Ok(self.get_with_name_mut(drive.as_bytes()))
    }

    /// gets the path in mountpoint from a given path
    /// basiclly removes the drive name
    pub(self) fn mountpoint_path(path: Path) -> String {
        let index = path.find(':').unwrap();
        let path_in_mountpoint = &path[index + 1..path.len()];
        path_in_mountpoint.to_string()
    }

    /// checks if a path is a vaild dir returns Err if path has an error
    pub fn verify_path_dir(&mut self, path: Path) -> FSResult<()> {
        let mountpoint = self
            .get_from_path(path)?
            .ok_or(FSError::NoSuchAFileOrDirectory)?;

        let res = mountpoint.reslove_path(path)?;

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

    fn reslove_path(&mut self, _path: Path) -> FSResult<&mut Inode> {
        unreachable!()
    }

    fn open(&mut self, path: Path) -> FSResult<FileDescriptor> {
        let mountpoint = self
            .get_from_path(path)?
            .ok_or(FSError::NoSuchAFileOrDirectory)?;

        let file = mountpoint.open(&Self::mountpoint_path(path))?;

        Ok(file)
    }

    fn read(&mut self, file_descriptor: &mut FileDescriptor, buffer: &mut [u8]) -> FSResult<usize> {
        unsafe { (*file_descriptor.mountpoint).read(file_descriptor, buffer) }
    }

    fn write(&mut self, file_descriptor: &mut FileDescriptor, buffer: &[u8]) -> FSResult<()> {
        unsafe { (*file_descriptor.mountpoint).write(file_descriptor, buffer) }
    }

    fn create(&mut self, path: Path, name: String) -> FSResult<()> {
        let mountpoint = self
            .get_from_path(path)?
            .ok_or(FSError::NoSuchAFileOrDirectory)?;

        mountpoint.create(path, name)
    }

    fn createdir(&mut self, path: Path, name: String) -> FSResult<()> {
        let mountpoint = self
            .get_from_path(path)?
            .ok_or(FSError::NoSuchAFileOrDirectory)?;

        mountpoint.createdir(path, name)
    }

    fn close(&mut self, file_descriptor: &mut FileDescriptor) -> FSResult<()> {
        unsafe { (*file_descriptor.mountpoint).close(file_descriptor) }
    }

    fn diriter_open(&mut self, fd: &mut FileDescriptor) -> FSResult<Box<dyn DirIter>> {
        unsafe { (*fd.mountpoint).diriter_open(fd) }
    }
}
