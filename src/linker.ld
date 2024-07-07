ENTRY(_start)

SECTIONS {
    . = 1M;
    .multiboot : {
        *(.multiboot)
    }

    .text : ALIGN(4K) {
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