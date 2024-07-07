.set STACK_SIZE, 16384
/* multiboot */
.set FLAG, 0x0
.set MAGIC, 0x1BADB002
.set CHECKSUM, -(MAGIC + FLAG)

.section .multiboot
.align 4
.long MAGIC
.long FLAG
.long CHECKSUM

.section .bss
.align 16
stack_bottom:
.skip STACK_SIZE
stack_top:

.section .text

.global _start
.type _start, @function

_start:
    mov $stack_top, %esp
    call kernel_main
    hlt