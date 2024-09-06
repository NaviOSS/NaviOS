use macros::test_module;

#[test_module]
pub mod testing_module {
    use alloc::vec::Vec;

    use crate::cross_println;
    use crate::println;
    use core::arch::asm;

    fn print() {
        assert_eq!(1, 1);
    }

    #[cfg(target_arch = "x86_64")]
    fn long_mode() {
        let rax: u64;
        unsafe {
            asm!(
                "
                    mov rax, 0xFFFFFFFFFFFFFFFF
                    mov {}, rax
                ",
                out(reg) rax
            );
        };

        assert_eq!(rax, 0xFFFFFFFFFFFFFFFF);
    }

    #[cfg(target_arch = "x86_64")]
    fn interrupts() {
        unsafe { asm!("int3") }
        assert_eq!(true, true);
    }

    fn allocator() {
        let mut test = Vec::new();

        for i in 0..100 {
            test.push(i);
        }

        println!("{:#?}\nAllocated Vec with len {}", test, test.len());
    }

    // syscall tests
    fn syscall() {
        unsafe { asm!("mov rax, 1; int 0x80") }
    }
}
