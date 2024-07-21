use crate::{
    memory::{
        frame_allocator::Frame,
        paging::{EntryFlags, Page},
    },
    paging_mapper, println, rsdp_addr,
};

fn map_present(addr: u64) {
    paging_mapper()
        .map_to(
            Page::containing_address(addr as usize),
            Frame::containing_address(addr as usize),
            EntryFlags::PRESENT,
        )
        .unwrap();
}

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
    pub signatrue: [u8; 4],
    len: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RSDT {
    pub header: ACPIHeader,
    table: [u32; 0], // uint32_t table[];?
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct XSDT {
    pub header: ACPIHeader,
    table: [u64; 0],
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct MADT {
    pub header: ACPIHeader,
    local_apic_address: u32,
    flags: u32,
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct MADTRecord {
    pub entry_type: u8,
    pub length: u8,
}

// any sdt
pub trait SDT {
    fn header(&self) -> &ACPIHeader;

    fn len(&self) -> u32 {
        self.header().len
    }

    unsafe fn nth(&self, n: usize) -> (usize, u32);
}

// RSDT and RSDT
// stands for Parent Table of System Descriptors (yes it gave me ptsd)
pub trait PTSD: SDT {
    // returns (ptr, offset)
    // offset can be used to iter
    // offset is the offset starting from the first byte of Self
    unsafe fn get_entry_of_signatrue(&self, signatrue: [u8; 4]) -> Option<*const ACPIHeader> {
        for i in 0..(self.count()) {
            let item = self.nth(i).0 as *const ACPIHeader;
            if (*item).signatrue == signatrue {
                return Some(item);
            }
        }
        None
    }

    // table item count
    fn count(&self) -> usize {
        (self.len() as usize - size_of::<ACPIHeader>()) / 4
    }
}

impl SDT for RSDT {
    fn header(&self) -> &ACPIHeader {
        &self.header
    }

    unsafe fn nth(&self, n: usize) -> (usize, u32) {
        let table_start = (self as *const Self).byte_add(size_of::<Self>());
        let offset = n * 4;

        let total_offset = (table_start as usize - (self as *const Self) as usize) + offset;
        let addr = *(table_start.byte_add(offset) as *const u32) as usize;
        map_present(addr as u64);

        (addr, total_offset as u32)
    }
}

impl PTSD for RSDT {}

impl SDT for MADT {
    fn header(&self) -> &ACPIHeader {
        &self.header
    }

    unsafe fn nth(&self, n: usize) -> (usize, u32) {
        let addr = self as *const Self;

        if n == 0 {
            let base = (addr).byte_add(size_of::<MADT>());
            return (base as usize, base as u32 - addr as u32);
        }

        let base = self.nth(0).0;
        let mut record = base as u32 + (*(base as *const MADTRecord)).length as u32;

        for _ in 1..n - 1 {
            let next_record = record as *const MADTRecord;
            let len = (*next_record).length;
            record += len as u32;
        }

        (record as usize, record as u32 - addr as u32)
    }
}

impl MADT {
    pub unsafe fn get_record_of_type(&self, ty: u8) -> Option<*const MADTRecord> {
        let len = self.header.len;
        let mut current_offset = 0;
        let mut i = 0;

        while current_offset <= len {
            let (ptr, offset) = self.nth(i);
            let ptr = ptr as *const MADTRecord;

            if (*ptr).entry_type == ty {
                return Some(ptr);
            }

            i += 1;
            current_offset = offset;
        }

        None
    }
}

fn get_rsdp() -> RSDPDesc {
    map_present(rsdp_addr());
    let ptr = rsdp_addr() as *mut RSDPDesc;

    let desc = unsafe { *ptr };
    println!("{:#?}", desc);
    desc
}

pub fn get_sdt() -> &'static dyn PTSD {
    let rsdp = get_rsdp();

    // if rsdp.xsdt_addr != 0 {
    //     map_present(rsdp.xsdt_addr);
    //     return SDT::XSDT(rsdp.xsdt_addr as *const XSDT);
    // }

    map_present(rsdp.rsdt_addr as u64);

    unsafe { &*(rsdp.rsdt_addr as *const RSDT) }
}
