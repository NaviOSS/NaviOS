use x86_64::{
    instructions::tables::load_tss,
    registers::segmentation::{Segment, CS, DS, ES, FS, GS, SS},
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

use lazy_static::lazy_static;

lazy_static! {
    pub static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[0] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = unsafe { &STACK.as_ptr() };
            let stack_end = unsafe { stack_start.add(STACK_SIZE) };
            VirtAddr::from_ptr(stack_end)
        };
        tss
    };
}

pub struct Selectors {
    code: SegmentSelector,
    data: SegmentSelector,
    tss: SegmentSelector,
}

lazy_static! {
    pub static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        let code = gdt.append(Descriptor::kernel_code_segment());
        let data = gdt.append(Descriptor::kernel_data_segment());
        let tss = gdt.append(Descriptor::tss_segment(&TSS));

        (gdt, Selectors { code, data, tss })
    };
}

pub fn init_gdt() {
    GDT.0.load();
    unsafe {
        let data_seg = GDT.1.data;
        SS::set_reg(data_seg);
        ES::set_reg(data_seg);
        FS::set_reg(data_seg);
        GS::set_reg(data_seg);
        DS::set_reg(data_seg);

        CS::set_reg(GDT.1.code);
        load_tss(GDT.1.tss);
    }
}
