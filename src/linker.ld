# OUTPUT_FORMAT("elf64-x86_64")
ENTRY(_start)


SECTIONS {
    . = 3M;

	.text : ALIGN(4K) {
        *(.multiboot)
		*(.text .text.*)
	}

	.rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
    }

	.data : ALIGN(4K) {
        *(.data .data.*)
    }

    .bss : ALIGN(4K) {
        *(.bss)
        *(COMMON)
    }

    PROVIDE(stack_bottom = . - 16384);
    PROVIDE(stack_top = stack_bottom + 16384);
}