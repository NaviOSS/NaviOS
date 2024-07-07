ENTRY(_start)

SECTIONS {
    . = 2M;
    .text : ALIGN(4K) {
        *(.multiboot)
        *(.text)
    }

    .rodata : ALIGN(4K) {
        *(.rodata)
    }

    .data : ALIGN(4K) {
        *(COMMON)
        *(.data)
    }
    
    .bss : ALIGN(4K) {
        *(.bss)
    }
}