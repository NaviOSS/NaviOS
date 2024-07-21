use core::{mem, ops::Index, slice};

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

#[derive(Debug)]
pub enum SDT {
    RSDT(*const RSDT),
    XSDT(*const XSDT),
}

impl RSDT {
    pub unsafe fn entries(&self) -> &[u32] {
        let num = (self.header.len as usize - size_of::<ACPIHeader>()) / 4;
        let entries = slice::from_raw_parts(self.table.as_ptr(), num);
        for entry in entries {
            map_present(*entry as u64);
        }

        entries
    }
}

fn get_rsdp() -> RSDPDesc {
    map_present(rsdp_addr());
    let ptr = rsdp_addr() as *mut RSDPDesc;

    let desc = unsafe { *ptr };
    println!("{:#?}", desc);
    desc
}

pub fn get_sdt() -> SDT {
    let rsdp = get_rsdp();

    // if rsdp.xsdt_addr != 0 {
    //     map_present(rsdp.xsdt_addr);
    //     return SDT::XSDT(rsdp.xsdt_addr as *const XSDT);
    // }

    map_present(rsdp.rsdt_addr as u64);

    SDT::RSDT(rsdp.rsdt_addr as *const RSDT)
}
