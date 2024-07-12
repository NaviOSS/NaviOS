use core::arch::asm;

use lazy_static::lazy_static;

use crate::terminal::framebuffer::kwriteln;
#[derive(Default)]
struct GDTEntry {
    base: u32,
    limit: u32,
    access: u8,
    flags: u8,
}

#[repr(C, packed)]
#[derive(Default)]
pub struct EncodedGDTEntry {
    limit0: u16,
    base0: u16,
    base1: u8,
    access: u8,
    limit1_flags: u8,
    base2: u8,
}

impl Into<EncodedGDTEntry> for GDTEntry {
    fn into(self) -> EncodedGDTEntry {
        let mut encoded = EncodedGDTEntry::default();

        encoded.limit0 = (self.limit & 0xFFFF) as u16;
        encoded.limit1_flags = ((self.limit >> 16) & 0x0F) as u8; // third limit byte
        encoded.limit1_flags |= self.flags & 0xF0; // first 4 bits

        encoded.base0 = (self.base & 0xFFFF) as u16;
        encoded.base1 = ((self.base >> 16) & 0xFF) as u8;
        encoded.base2 = ((self.base >> 24) & 0xFF) as u8;

        encoded.access = self.access;
        encoded
    }
}

const ACCESS_ACCESSED: u8 = 1 << 0;
const ACCESS_WRITE_READ: u8 = 1 << 1;
const ACCESS_DIR_DOWN: u8 = 1 << 2;
const ACCESS_EXECUTABLE: u8 = 1 << 3;
const NON_SYSTEM: u8 = 1 << 4;
const RING1: u8 = 1 << 5;
const RING2: u8 = 1 << 6;
const RING3: u8 = RING1 as u8 | RING2 as u8;
const ACCESS_VAILD: u8 = 1 << 7;

const FLAG_LONG: u8 = 1 << 5;
const FLAG_IS32BIT: u8 = 1 << 6;
const FLAG_PAGELIMIT: u8 = 1 << 7;

pub type GDTType = [EncodedGDTEntry; 3];

lazy_static! {
    pub static ref GDT: GDTType = [
        GDTEntry::default().into(),
        GDTEntry {
            base: 0,
            limit: 0xFFFFF,
            access: ACCESS_VAILD | NON_SYSTEM | ACCESS_WRITE_READ | ACCESS_EXECUTABLE,
            flags: FLAG_PAGELIMIT | FLAG_LONG
        }
        .into(), // kernel code segment
        GDTEntry {
            base:0,
            limit: 0xFFFFF,
            access: ACCESS_VAILD | ACCESS_WRITE_READ | NON_SYSTEM,
            flags: FLAG_PAGELIMIT | FLAG_LONG
        }.into(), // kernel data segment
    ];
}
#[repr(C, packed)]
pub struct GDTDescriptor {
    pub limit: u16,
    pub base: u32,
}
lazy_static! {
    pub static ref GDT_DESCRIPTOR: GDTDescriptor = GDTDescriptor {
        limit: (size_of::<GDTType>() - 1) as u16,
        base: (&*GDT) as *const GDTType as u32
    };
}

pub fn init_gdt() {
    unsafe {
        asm!("lgdt [{}]", in(reg) &*GDT_DESCRIPTOR, options(nostack));
    }
    kwriteln("loaded gdt using lgdt sucess i think....");
}
