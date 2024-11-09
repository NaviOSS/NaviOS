use alloc::{boxed::Box, vec::Vec};

use crate::drivers::vfs::{expose::DirIter, FileDescriptor, FS, VFS_STRUCT};

pub enum Resource {
    Null,
    File(FileDescriptor),
    /// TODO: better diriter implementation
    DirIter(Box<dyn DirIter>),
}

impl Clone for Resource {
    fn clone(&self) -> Self {
        match self {
            Self::Null => Self::Null,
            Self::File(ref fd) => Self::File(fd.clone()),
            Self::DirIter(ref diriter) => Self::DirIter(DirIter::clone(&**diriter)),
        }
    }
}
impl Resource {
    pub const fn variant(&self) -> u8 {
        match self {
            Resource::Null => 0,
            Resource::File(_) => 1,
            Resource::DirIter(_) => 2,
        }
    }
}

pub struct ResourceManager {
    resources: Vec<Resource>,
    next_ri: usize,
}

impl ResourceManager {
    pub fn new() -> Self {
        ResourceManager {
            resources: Vec::with_capacity(2),
            next_ri: 0,
        }
    }

    pub fn add_resource(&mut self, resource: Resource) -> usize {
        let resources = &mut self.resources[self.next_ri..];

        for (mut ri, res) in resources.iter_mut().enumerate() {
            if res.variant() == Resource::Null.variant() {
                ri += self.next_ri;

                self.next_ri = ri;
                *res = resource;

                return ri;
            }
        }

        self.resources.push(resource);

        let ri = self.resources.len() - 1;
        self.next_ri = ri;

        ri
    }

    #[inline]
    pub fn remove_resource(&mut self, ri: usize) -> Result<(), ()> {
        if ri >= self.resources.len() {
            return Err(());
        }

        self.resources[ri] = Resource::Null;
        if ri < self.next_ri {
            self.next_ri = ri;
        }
        Ok(())
    }

    /// cleans up all resources
    /// returns the **previous** next resource index
    pub fn clean(&mut self) -> usize {
        for resource in &mut self.resources {
            match resource {
                Resource::File(fd) => VFS_STRUCT.read().close(fd).unwrap(),
                _ => *resource = Resource::Null,
            }
        }

        let prev = self.next_ri;
        self.next_ri = 0;
        prev
    }

    pub fn next_ri(&self) -> usize {
        self.next_ri
    }

    pub fn overwrite_resources(&mut self, resources: Vec<Resource>) {
        self.resources = resources;
    }

    pub fn clone_resources(&self) -> Vec<Resource> {
        self.resources.clone()
    }

    /// gets a mutable reference to the resource with index `ri`
    /// returns `None` if `ri` is invaild
    pub fn get(&mut self, ri: usize) -> Option<&mut Resource> {
        let resources = &mut self.resources;

        if ri >= resources.len() {
            return None;
        }

        Some(&mut resources[ri])
    }
}
