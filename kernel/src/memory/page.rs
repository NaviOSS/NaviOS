use bitflags::bitflags;

#[cfg(target_arch = "x86_64")]
bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct PageTableFlags: u64 {
        const PRESENT =         1;

        const WRITABLE =        1 << 1;

        const USER_ACCESSIBLE = 1 << 2;

        const WRITE_THROUGH =   1 << 3;
        const NO_CACHE =        1 << 4;

        const ACCESSED =        1 << 5;

        const DIRTY =           1 << 6;

        const HUGE_PAGE =       1 << 7;

        const GLOBAL =          1 << 8;

        const BIT_9 =           1 << 9;
        const BIT_10 =          1 << 10;
        const BIT_11 =          1 << 11;
        const BIT_52 =          1 << 52;
        const BIT_53 =          1 << 53;
        const BIT_54 =          1 << 54;
        const BIT_55 =          1 << 55;
        const BIT_56 =          1 << 56;
        const BIT_57 =          1 << 57;
        const BIT_58 =          1 << 58;
        const BIT_59 =          1 << 59;
        const BIT_60 =          1 << 60;
        const BIT_61 =          1 << 61;
        const BIT_62 =          1 << 62;
        const NO_EXECUTE =      1 << 63;
    }
}
