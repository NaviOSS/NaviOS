pub mod expose;

use core::usize;

use crate::{
    debug,
    threading::expose::{getcwd, ErrorStatus},
    utils::{
        ustar::{self, TarArchiveIter},
        Locked,
    },
};
pub mod devicefs;
pub mod ramfs;

use alloc::{
    borrow::ToOwned,
    boxed::Box,
    collections::btree_map::BTreeMap,
    string::{String, ToString},
    sync::Arc,
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
    vfs.mount(b"dev", Box::new(devicefs::DeviceFS::new()))
        .unwrap();
    debug!(VFS, "done ...");
}

#[derive(Clone)]
pub struct FileDescriptor {
    pub mountpoint: *mut dyn FS,
    pub node: Inode,
    /// acts as a dir entry index for directories
    /// acts as a byte index for files
    pub read_pos: usize,
    /// acts as a byte index for files
    /// doesn't do anything for directories
    pub write_pos: usize,
}

impl FileDescriptor {
    pub fn new<'a>(mountpoint: *mut dyn FS, node: Inode) -> Self {
        Self {
            mountpoint,
            node,
            read_pos: 0,
            write_pos: 0,
        }
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
    AlreadyExists,
    NotExecuteable,
}
impl Into<ErrorStatus> for FSError {
    fn into(self) -> ErrorStatus {
        match self {
            Self::OperationNotSupported => ErrorStatus::OperationNotSupported,
            Self::NotAFile => ErrorStatus::NotAFile,
            Self::NotADirectory => ErrorStatus::NotADirectory,
            Self::NoSuchAFileOrDirectory => ErrorStatus::NoSuchAFileOrDirectory,
            Self::InvaildPath => ErrorStatus::InvaildPath,
            Self::InvaildDrive => ErrorStatus::InvaildDrive,
            Self::InvaildFileDescriptorOrRes => ErrorStatus::InvaildResource,
            Self::AlreadyExists => ErrorStatus::AlreadyExists,
            Self::NotExecuteable => ErrorStatus::NotExecutable,
        }
    }
}
pub type FSResult<T> = Result<T, FSError>;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InodeType {
    File,
    Directory,
    Device,
}
pub trait InodeOps: Send + Sync {
    fn name(&self) -> String;
    /// gets an Inode from self
    /// returns Err(()) if self is not a directory
    /// returns Ok(None) if self doesn't contain `name`
    /// returns Ok(inodeid) if successful
    fn get(&self, name: &str) -> FSResult<Option<usize>> {
        _ = name;
        FSResult::Err(FSError::OperationNotSupported)
    }
    /// checks if node contains `name` returns false if it doesn't or if it is not a directory
    fn contains(&self, name: &str) -> bool {
        _ = name;
        false
    }
    /// returns the size of node
    /// different nodes may use this differently but in case it is a normal file it will always give the
    /// file size in bytes
    fn size(&self) -> FSResult<usize> {
        Err(FSError::OperationNotSupported)
    }
    /// attempts to read `count` bytes of node data if it is a file
    /// panics if invaild `offset`
    /// returns the amount of bytes read
    fn read(&self, buffer: &mut [u8], offset: usize, count: usize) -> FSResult<usize> {
        _ = buffer;
        _ = offset;
        _ = count;
        Err(FSError::OperationNotSupported)
    }
    /// attempts to write `buffer.len` bytes from `buffer` into node data if it is a file starting
    /// from offset
    /// extends the nodes data and node size if `buffer.len` + `offset` is greater then node size
    /// returns the amount of bytes written
    fn write(&self, buffer: &[u8], offset: usize) -> FSResult<usize> {
        _ = buffer;
        _ = offset;
        Err(FSError::OperationNotSupported)
    }

    /// attempts to insert a node to self
    /// returns an FSError::NotADirectory if not a directory
    fn insert(&self, name: &str, node: usize) -> FSResult<()> {
        _ = name;
        _ = node;
        Err(FSError::OperationNotSupported)
    }

    fn inodeid(&self) -> usize;
    fn kind(&self) -> InodeType;

    #[inline(always)]
    fn is_dir(&self) -> bool {
        self.kind() == InodeType::Directory
    }
}

/// unknown inode type
pub type Inode = Arc<dyn InodeOps>;
/// inode type with a known type
pub type InodeOf<T> = Arc<T>;

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

    fn get_inode(&self, inode_id: usize) -> FSResult<Option<Inode>> {
        _ = inode_id;
        Err(FSError::OperationNotSupported)
    }

    #[inline]
    fn root_inode(&self) -> FSResult<Inode> {
        Ok(self.get_inode(0)?.unwrap())
    }

    /// goes trough path to get the inode it refers to
    /// will err if there is no such a file or directory or path is straight up invaild
    fn reslove_path(&mut self, path: Path) -> FSResult<Inode> {
        let mut path = path.split(&['/', '\\']).peekable();

        let mut current_inode = self.root_inode()?;

        if path.peek() == Some(&"") {
            path.next();
        }

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

            if !current_inode.is_dir() {
                return Err(FSError::NoSuchAFileOrDirectory);
            }

            if !current_inode.contains(depth) {
                return Err(FSError::NoSuchAFileOrDirectory);
            }

            let inodeid = current_inode.get(depth)?.unwrap();
            current_inode = self.get_inode(inodeid)?.unwrap();
        }

        return Ok(current_inode.clone());
    }

    /// goes trough path to get the inode it refers to
    /// will err if there is no such a file or directory or path is straight up invaild
    /// assumes that the last depth in path is the filename and returns it alongside the parent dir
    fn reslove_path_uncreated<'a>(&mut self, path: Path<'a>) -> FSResult<(Inode, &'a str)> {
        let path = path.trim_end_matches('/');

        let (name, path) = {
            let beginning = path.bytes().rposition(|c| c == b'/');

            if let Some(idx) = beginning {
                (&path[idx + 1..], &path[..idx])
            } else {
                (path, "/")
            }
        };

        let resloved = self.reslove_path(path)?;
        if resloved.kind() != InodeType::Directory {
            return Err(FSError::NotADirectory);
        }

        Ok((resloved, name))
    }

    /// opens a path returning a file descriptor or an Err(()) if path doesn't exist
    fn open(&mut self, path: Path) -> FSResult<FileDescriptor> {
        _ = path;
        Err(FSError::OperationNotSupported)
    }
    /// attempts to read `buffer.len` bytes from file_descriptor returns the actual count of the bytes read
    /// shouldn't read directories!
    fn read(&mut self, file_descriptor: &mut FileDescriptor, buffer: &mut [u8]) -> FSResult<usize> {
        _ = file_descriptor;
        _ = buffer;
        Err(FSError::OperationNotSupported)
    }
    /// attempts to write `buffer.len` bytes to `file_descriptor`
    /// shouldn't write to directories!
    fn write(&mut self, file_descriptor: &mut FileDescriptor, buffer: &[u8]) -> FSResult<usize> {
        _ = file_descriptor;
        _ = buffer;
        Err(FSError::OperationNotSupported)
    }
    /// creates an empty file named `name` in `path`
    fn create(&mut self, path: Path) -> FSResult<()> {
        _ = path;
        Err(FSError::OperationNotSupported)
    }
    /// creates an empty dir named `name` in `path`
    fn createdir(&mut self, path: Path) -> FSResult<()> {
        _ = path;
        Err(FSError::OperationNotSupported)
    }

    /// opens an iterator of directroy entires, fd must be a directory
    fn diriter_open(&mut self, fd: &mut FileDescriptor) -> FSResult<Box<dyn DirIter>> {
        _ = fd;
        Err(FSError::OperationNotSupported)
    }
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

    /// gets a drive from `self` named "`name`"
    /// or "`name`:" muttabily
    pub(self) fn get_with_name_mut(&mut self, name: &[u8]) -> Option<&mut Box<dyn FS>> {
        let mut name = name;

        if name.ends_with(b":") {
            name = &name[..name.len() - 1];
        }

        self.drivers.get_mut(name)
    }

    /// gets the drive name from `path` then gets the drive
    /// path must be absolute starting with DRIVE_NAME:/
    /// returns an &str which removes the DRIVE_NAME:/ from path
    /// also handles relative path
    pub(self) fn get_from_path(&mut self, path: Path) -> FSResult<(&mut Box<dyn FS>, String)> {
        let mut spilt_path = path.split(&['/', '\\']);

        let drive = spilt_path.next().ok_or(FSError::InvaildDrive)?;
        if !(drive.ends_with(':')) {
            let full_path = getcwd().to_owned() + path;
            return self.get_from_path_checked(&full_path);
        }

        Ok((
            self.get_with_name_mut(drive.as_bytes())
                .ok_or(FSError::InvaildDrive)?,
            path[drive.len()..].to_string(),
        ))
    }
    /// get_from_path but path cannot be realtive to cwd
    pub(self) fn get_from_path_checked(
        &mut self,
        path: Path,
    ) -> FSResult<(&mut Box<dyn FS>, String)> {
        let mut spilt_path = path.split(&['/', '\\']);

        let drive = spilt_path.next().ok_or(FSError::InvaildDrive)?;
        if !(drive.ends_with(':')) {
            return Err(FSError::InvaildDrive);
        }

        Ok((
            self.get_with_name_mut(drive.as_bytes())
                .ok_or(FSError::InvaildDrive)?,
            path[drive.len()..].to_string(),
        ))
    }

    /// checks if a path is a vaild dir returns Err if path has an error
    pub fn verify_path_dir(&mut self, path: Path) -> FSResult<()> {
        let (mountpoint, path) = self.get_from_path_checked(path)?;

        let res = mountpoint.reslove_path(&path)?;

        if !res.is_dir() {
            return Err(FSError::NotADirectory);
        }
        Ok(())
    }

    pub fn unpack_tar(fs: &mut dyn FS, tar: &mut TarArchiveIter) -> FSResult<()> {
        while let Some(inode) = tar.next() {
            let path = inode.name();

            match inode.kind {
                ustar::Type::NORMAL => {
                    fs.create(path)?;

                    let mut opened = fs.open(path)?;
                    fs.write(&mut opened, inode.data())?;
                    fs.close(&mut opened)?;
                }

                ustar::Type::DIR => fs.createdir(path.trim_end_matches('/'))?,

                _ => return Err(FSError::OperationNotSupported),
            };
        }
        Ok(())
    }
}

impl FS for VFS {
    fn name(&self) -> &'static str {
        "vfs"
    }

    fn open(&mut self, path: Path) -> FSResult<FileDescriptor> {
        let (mountpoint, path) = self.get_from_path(path)?;

        let file = mountpoint.open(&path)?;

        Ok(file)
    }

    fn read(&mut self, file_descriptor: &mut FileDescriptor, buffer: &mut [u8]) -> FSResult<usize> {
        unsafe { (*file_descriptor.mountpoint).read(file_descriptor, buffer) }
    }

    fn write(&mut self, file_descriptor: &mut FileDescriptor, buffer: &[u8]) -> FSResult<usize> {
        unsafe { (*file_descriptor.mountpoint).write(file_descriptor, buffer) }
    }

    fn create(&mut self, path: Path) -> FSResult<()> {
        let (mountpoint, path) = self.get_from_path(path)?;

        if path.ends_with('/') {
            return Err(FSError::NotAFile);
        }

        mountpoint.create(&path)
    }

    fn createdir(&mut self, path: Path) -> FSResult<()> {
        let (mountpoint, path) = self.get_from_path(path)?;

        mountpoint.createdir(&path)
    }

    fn close(&mut self, file_descriptor: &mut FileDescriptor) -> FSResult<()> {
        unsafe { (*file_descriptor.mountpoint).close(file_descriptor) }
    }

    fn diriter_open(&mut self, fd: &mut FileDescriptor) -> FSResult<Box<dyn DirIter>> {
        unsafe { (*fd.mountpoint).diriter_open(fd) }
    }
}
