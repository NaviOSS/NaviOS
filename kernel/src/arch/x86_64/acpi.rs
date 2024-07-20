use core::{ops::Index, panicking::panic};

use crate::rsdp_addr;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct RSDPDesc {
    signature: [u8; 8],
    checksum: u8,
    oemid: [u8; 6],
    revision: u8,
    rsdt_addr: u32,
    len: u32,
    xsdt_addr: u64,
    extended_checksum: u8,
    reserved: [u8; 3],
}

impl RSDPDesc {
    pub fn vaildate(&self) -> bool {
        let size = size_of::<Self>();
        let byte_array = (self) as *const RSDPDesc as *const u8;
        let mut sum: usize = 0;

        for i in 0..size {
            unsafe {
                sum += *byte_array.add(i) as usize;
            };
        }

        (sum & 0xFF) == 0
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct ACPIHeader {
    signatrue: [u8; 4],
    len: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    creator_id: u32,
    creator_revision: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct RSDT {
    pub header: ACPIHeader,
    table: [u32; 0], // uint32_t table[];?
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct XSDT {
    pub header: ACPIHeader,
    table: [u64; 0], // uint32_t table[];?
}

#[derive(Debug)]
pub enum SDT {
    RSDT(*const RSDT),
    XSDT(*const XSDT),
}

impl SDT {
    pub fn table_len(&self) -> usize {
        let bytes = self.table_bytes();

        let len = unsafe {
            match self {
                SDT::RSDT(ptr) => (**ptr).header.len,
                SDT::XSDT(ptr) => (**ptr).header.len,
            }
        };

        (len as usize - size_of::<ACPIHeader>()) / bytes
    }

    #[inline]
    pub fn table_bytes(&self) -> usize {
        match self {
            SDT::RSDT(_) => 4,
            SDT::XSDT(_) => 8,
        }
    }

    pub fn nth(&self, index: usize) -> *const ACPIHeader {
        let ptr = unsafe {
            match self {
                SDT::RSDT(ptr) => (*ptr).add(size_of::<ACPIHeader>()) as *const u8,
                SDT::XSDT(ptr) => (*ptr).add(size_of::<ACPIHeader>()) as *const u8,
            }
        };

        if self.table_len() <= index {
            panic!(
                "index out of bounds: the len is {} but the index is {} while trying to index an RSDT or an XSDT",
                self.table_len(),
                index
            );
        }

        let item = unsafe { *(ptr.add(self.table_bytes() * index)) as usize };
        item as *const ACPIHeader
    }
}

impl Index<usize> for SDT {
    type Output = ACPIHeader;
    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &(*self.nth(index)) }
    }
}

fn get_rsdp() -> RSDPDesc {
    let ptr = rsdp_addr() as *mut RSDPDesc;
    unsafe { *ptr }
}

fn get_sdt() -> SDT {
    let rsdp = get_rsdp();

    if rsdp.xsdt_addr != 0 {
        return SDT::RSDT(rsdp.xsdt_addr as *const RSDT);
    }

    SDT::XSDT(rsdp.rsdt_addr as *const XSDT)
}
