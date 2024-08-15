use macros::test_module;

#[test_module]
pub mod testing_module {
    use alloc::vec::Vec;

    use crate::{global_allocator, println};
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

    // TODO: add asserts for the extend_test
    fn extending_the_heap() {
        global_allocator()
            .lock()
            .extend_heap()
            .unwrap_or_else(|_| panic!());
        println!("extended the heap successfully!");
    }

    fn double_extending_the_heap() {
        global_allocator()
            .lock()
            .extend_heap()
            .unwrap_or_else(|_| panic!());
        global_allocator()
            .lock()
            .extend_heap()
            .unwrap_or_else(|_| panic!());

        println!("double extended the heap successfully!");
    }
}
