use crate::{arch::x86_64::inw, kernel, memory::identity_map_present, serial};

use super::outb;

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

// #[repr(C)]
// #[derive(Debug, Clone, Copy)]
// pub struct XSDT {
//     pub header: ACPIHeader,
//     table: [u64; 0],
// }

#[repr(C, packed)]
#[derive(Debug)]
pub struct FADT {
    pub header: ACPIHeader,
    pub firmware_ctrl: u32,
    pub dsdt: u32,
    pub reserved: u8,
    pub preferred_pm_profile: u8,

    pub sci_int: u16,
    pub smi_cmd: u32,

    pub acpi_enable: u8,
    pub acpi_disable: u8,

    pub s4bios_req: u8,
    pub pstate_cnt: u8,

    pub pm1a_evt_blk: u32,
    pub pm1b_evt_blk: u32,
    pub pm1a_cnt_blk: u32,
    pub pm1b_cnt_blk: u32,

    pub pm2_cnt_blk: u32,
    pub pm_tmr_blk: u32,

    pub gpe0_blk: u32,
    pub gpe1_blk: u32,

    pub pm1_evt_len: u8,
    pub pm1_cnt_len: u8,
    pub pm2_cnt_len: u8,
    pub pm_tmr_len: u8,

    pub gpe0_blk_len: u8,
    pub gpe1_blk_len: u8,
    pub gpe1_base: u8,

    pub cst_cnt: u8,

    pub p_lvl2_lat: u16,
    pub p_lvl3_lat: u16,

    pub flush_size: u16,
    pub flush_stride: u16,

    pub duty_offset: u8,
    pub duty_width: u8,

    pub day_alrm: u8,
    pub mon_alrm: u8,

    pub century: u8,
    pub iapc_boot_arch: u16,
    pub reserved2: u8,
    pub flags: u32,
    pub reset_reg: GenericAddressStructure,
    pub reset_value: u8,
    pub arm_boot_arch: u16,
    pub fadt_minor_version: u8,

    pub x_firmware_ctrl: u64,
    pub x_dsdt: u64,

    pub x_pm1a_evt_blk: GenericAddressStructure,
    pub x_pm1b_evt_blk: GenericAddressStructure,
    pub x_pm1a_cnt_blk: GenericAddressStructure,
    pub x_pm1b_cnt_blk: GenericAddressStructure,
    pub x_pm2_cnt_blk: GenericAddressStructure,
    pub x_pm_tmr_blk: GenericAddressStructure,

    pub x_gpe0_blk: GenericAddressStructure,
    pub x_gpe1_blk: GenericAddressStructure,

    pub sleep_control_reg: GenericAddressStructure,
    pub sleep_status_reg: GenericAddressStructure,

    pub hypervisor_vendor_id: u64,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct GenericAddressStructure {
    pub address_space: u8,
    pub bit_width: u8,
    pub bit_offset: u8,
    pub access_size: u8,
    pub address: u64,
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct MADT {
    pub header: ACPIHeader,
    local_apic_address: u32,
    flags: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
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
        identity_map_present(addr);

        (addr, total_offset as u32)
    }
}

impl PTSD for RSDT {}

impl SDT for FADT {
    fn header(&self) -> &ACPIHeader {
        &self.header
    }

    unsafe fn nth(&self, _: usize) -> (usize, u32) {
        panic!("FADT SDT doesn't support nth!")
    }
}

impl FADT {
    pub fn get(ptsd: &dyn PTSD) -> &FADT {
        unsafe { &*(ptsd.get_entry_of_signatrue(*b"FACP").unwrap() as *const FADT) }
    }
}

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

    pub fn get(ptsd: &dyn PTSD) -> &MADT {
        unsafe { &*(ptsd.get_entry_of_signatrue(*b"APIC").unwrap() as *const MADT) }
    }
}

fn get_rsdp() -> RSDPDesc {
    identity_map_present(kernel().rsdp_addr.unwrap() as usize);
    let ptr = kernel().rsdp_addr.unwrap() as *mut RSDPDesc;

    let desc = unsafe { *ptr };
    desc
}

pub fn get_sdt() -> &'static dyn PTSD {
    let rsdp = get_rsdp();

    // if rsdp.xsdt_addr != 0 {
    //     map_present(rsdp.xsdt_addr);
    //     return SDT::XSDT(rsdp.xsdt_addr as *const XSDT);
    // }

    identity_map_present(rsdp.rsdt_addr as usize);

    unsafe { &*(rsdp.rsdt_addr as *const RSDT) }
}

/// enable the acpi if not already enabled
pub fn enable_acpi(fadt: &FADT) {
    if !(fadt.smi_cmd == 0
        || ((fadt.acpi_enable == fadt.acpi_disable) && fadt.acpi_disable == 0)
        || inw(fadt.pm1a_cnt_blk as u16) & 1 == 1)
    {
        serial!(
            "enabling the acpi... smi: 0x{:X}, enable: 0x{:X}\n",
            fadt.smi_cmd as u16,
            fadt.acpi_enable
        );
        outb(fadt.smi_cmd as u16, fadt.acpi_enable);

        while (inw(fadt.smi_cmd as u16) & 1) == 0 {
            serial!("stuff\n")
        }

        if (inw(fadt.pm1a_evt_blk as u16) & 1) == 0 {
            panic!("failed to enable acpi");
        }
    }
}
