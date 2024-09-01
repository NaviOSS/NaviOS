global _start
section .text
_start:
	int 0x80
	jmp loop
loop:
	hlt
	jmp loop
