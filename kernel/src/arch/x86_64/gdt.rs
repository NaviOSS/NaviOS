use core::arch::asm;

use lazy_static::lazy_static;

use crate::println;

#[repr(C, packed)]
pub struct GDTEntry {
    limit0: u16,
    base0: u16,
    base1: u8,
    access: u8,
    limit1_flags: u8,
    base2: u8,
}

impl GDTEntry {
    const fn default() -> Self {
        Self {
            limit0: 0,
            base0: 0,
            base1: 0,
            access: 0,
            limit1_flags: 0,
            base2: 0,
        }
    }

    const fn new(base: u32, limit: u32, access: u8, flags: u8) -> Self {
        let mut encoded = Self::default();

        encoded.limit0 = (limit & 0xFFFF) as u16;
        encoded.limit1_flags = ((limit >> 16) & 0x0F) as u8; // third limit byte
        encoded.limit1_flags |= flags & 0xF0; // first 4 bits

        encoded.base0 = (base & 0xFFFF) as u16;
        encoded.base1 = ((base >> 16) & 0xFF) as u8;
        encoded.base2 = ((base >> 24) & 0xFF) as u8;

        encoded.access = access;
        encoded
    }

    const fn new_upper_64seg(base: u64) -> Self {
        let mut encoded = Self::default();
        let base = (base >> 32) as u32;

        encoded.limit0 = (base & 0xFFFF) as u16;
        encoded.base0 = ((base >> 16) & 0xFFFF) as u16;
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

const ACCESS_TYPE_TSS: u8 = 0x9;

const FLAG_LONG: u8 = 1 << 5;
const FLAG_IS32BIT: u8 = 1 << 6;
const FLAG_PAGELIMIT: u8 = 1 << 7;

// TSS setup
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct TaskStateSegment {
    reserved_1: u32,
    pub privilege_stack_table: [u64; 3],
    reserved_2: u64,
    pub interrupt_stack_table: [u64; 7],
    reserved_3: u64,
    reserved_4: u16,
    pub iomap_base: u16,
}

impl TaskStateSegment {
    pub const fn new() -> Self {
        Self {
            reserved_1: 0,
            privilege_stack_table: [0u64; 3],
            reserved_2: 0,
            interrupt_stack_table: [0u64; 7],
            reserved_3: 0,
            reserved_4: 0,
            iomap_base: 0,
        }
    }
}

lazy_static! {
    pub static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[0] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = unsafe { &STACK.as_ptr() };
            let stack_end = unsafe { stack_start.add(STACK_SIZE) };
            stack_end as u64
        };
        tss
    };
}
pub type GDTType = [GDTEntry; 5];

lazy_static! {
    pub static ref GDT: GDTType = [
        GDTEntry::default().into(),
        GDTEntry::new(
            0,
            0xFFFFF,
            ACCESS_VAILD | NON_SYSTEM | ACCESS_WRITE_READ | ACCESS_EXECUTABLE,
            FLAG_PAGELIMIT | FLAG_LONG
        ), // kernel code segment
        GDTEntry::new(
            0,
            0xFFFFF,
            ACCESS_VAILD | ACCESS_WRITE_READ | NON_SYSTEM,
            FLAG_PAGELIMIT | FLAG_LONG
        ), // kernel data segment

        GDTEntry::new(
            (((&*TSS) as *const TaskStateSegment as u64) & 0xFFFFFFFF) as u32,
            (size_of::<TaskStateSegment>() - 1) as u32,
            ACCESS_VAILD | ACCESS_TYPE_TSS,
            FLAG_PAGELIMIT | FLAG_LONG
        ), // TSS segment
        GDTEntry::new_upper_64seg(
            &*TSS as *const TaskStateSegment as u64,
        )
    ];
}
#[repr(C, packed)]
pub struct GDTDescriptor {
    pub limit: u16,
    pub base: usize,
}
lazy_static! {
    pub static ref GDT_DESCRIPTOR: GDTDescriptor = GDTDescriptor {
        limit: (size_of::<GDTType>() - 1) as u16,
        base: (&*GDT) as *const GDTType as usize
    };
}

pub fn init_gdt() {
    unsafe {
        asm!("lgdt [{}]", in(reg) &*GDT_DESCRIPTOR, options(nostack));

        asm!(
            "
            mov ax, 0x10
            mov gs, ax
            mov fs, ax
            mov ds, ax
            mov es, ax
            mov ss, ax
        "
        );

        asm!(
            "            
            push 0x08
            lea rax, [rip + 2f]
            push rax
            retfq
            2:
            ",
            options(nostack),
        );

        asm!("ltr {0:x}", in(reg) ((3*8) as u16))
    }
}
