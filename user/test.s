global _start
section .text
_start:
	int i
	jmp loop
loop:
	hlt
	jmp loop
section .data
i DB 0x80
