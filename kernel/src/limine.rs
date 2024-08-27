use alloc::slice;
use lazy_static::lazy_static;
use limine::file::File;
use limine::framebuffer::MemoryModel;
use limine::request::FramebufferRequest;
use limine::request::HhdmRequest;
use limine::request::KernelAddressRequest;
use limine::request::KernelFileRequest;
use limine::request::MemoryMapRequest;
use limine::request::RsdpRequest;

use limine::response::MemoryMapResponse;
use limine::BaseRevision;

use crate::memory::align_up;
use crate::terminal::framebuffer::FrameBufferInfo;
use crate::terminal::framebuffer::PixelFormat;

#[used]
#[link_section = ".requests"]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[link_section = ".requests"]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
#[link_section = ".requests"]
static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();

#[used]
#[link_section = ".requests"]
static KERNEL_ADDRESS_REQUEST: KernelAddressRequest = KernelAddressRequest::new();

#[used]
#[link_section = ".requests"]
static KERNEL_FILE_REQUEST: KernelFileRequest = KernelFileRequest::new();

#[used]
#[link_section = ".requests"]
static MMAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

#[used]
#[link_section = ".requests"]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

pub fn get_phy_offset() -> usize {
    HHDM_REQUEST.get_response().unwrap().offset() as usize
}

pub fn rsdp_addr() -> Option<u64> {
    Some(RSDP_REQUEST.get_response()?.address() as u64)
}

pub fn kernel_file() -> &'static File {
    KERNEL_FILE_REQUEST.get_response().unwrap().file()
}

pub fn kernel_image_info() -> (usize, usize, usize) {
    let addr = KERNEL_ADDRESS_REQUEST.get_response().unwrap();
    let size = kernel_file().size() as usize;

    (
        addr.physical_base() as usize,
        addr.virtual_base() as usize,
        size,
    )
}

pub fn mmap_request() -> &'static MemoryMapResponse {
    MMAP_REQUEST.get_response().unwrap()
}

lazy_static! {
    pub static ref MEMORY_SIZE: usize = {
        let mut physical_memory_size = 0;

        for entry in mmap_request().entries() {
            physical_memory_size += entry.length as usize;
        }
        physical_memory_size
    };
    pub static ref MEMORY_END: usize = {
        let mut largest_addr = 0;
        for entry in mmap_request().entries() {
            let end = (entry.base + entry.length) as usize;

            if end > largest_addr {
                largest_addr = end;
            }
        }

        core::cmp::max(0x0000_0001_0000_0000, largest_addr)
    };
}

pub fn get_phy_offset_end() -> usize {
    get_phy_offset() + *MEMORY_END
}

pub fn get_framebuffer() -> (&'static mut [u8], FrameBufferInfo) {
    let mut buffers = FRAMEBUFFER_REQUEST.get_response().unwrap().framebuffers();
    let first = buffers.next().unwrap();

    let pixel_format = match first.memory_model() {
        MemoryModel::RGB => PixelFormat::Rgb,
        _ => panic!("unknown limine framebuffer format"),
    };

    let bytes_per_pixel = align_up(first.bpp() as usize, 8) / 8;
    let info = FrameBufferInfo {
        bytes_per_pixel,
        stride: first.pitch() as usize / bytes_per_pixel,
        pixel_format,
    };

    assert_eq!(info.bytes_per_pixel, 4);

    let size = (first.width() * first.height() * first.bpp() as u64 / 8) as usize;
    let buffer = unsafe { slice::from_raw_parts_mut(first.addr(), size) };

    (buffer, info)
}
