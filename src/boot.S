.set STACK_SIZE, 16384

.section .boot

.section .bss
.align 16
stack_bottom:
.skip STACK_SIZE
stack_top:

.section .text
.global _start
.type _start, %function

_start:
    ldr sp, =stack_top
    bl kernel_main
    b .
    