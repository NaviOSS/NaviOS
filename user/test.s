global _start
section .text
_start:
	mov rax, 1
	int 0x80 

	; exit
	mov rax, 0
	int 0x80
