global _start

section .text
_start:
	; prompt 1
	mov rax, 3
	mov rdi, 1
	mov rsi, msg
	mov rdx, len
	int 0x80 
	
	; reading data
	mov rax, 4
	mov rdi, 0
	mov rsi, data
	mov rdx, 3
	int 0x80
	
	; writing some useless stuff
	mov rax, 3
	mov rdi, 1
	mov rsi, msg2
	mov rdx, len2
	int 0x80
	
	; writing back the data
	mov rax, 3
	mov rdi, 1
	mov rsi, data
	mov rdx, 4
	int 0x80 
	
	; testing errors	
	mov rax, 3
	mov rdi, 1
	mov rsi, msg3
	mov rdx, len3
	int 0x80
	
	; lets go!
	mov rax, 3
	mov rdi, 0
	mov rsi, data
	mov rdx, 4
	int 0x80
	
	; lets write rax
	neg rax
	add rax, '0'
	mov [data], rax
	; first we write the msg
	mov rax, 3
	mov rdi, 1
	mov rsi, msg4
	mov rdx, len4
	int 0x80 ; too much fun!

	; now we write data
	mov rax, 3
	mov rdi, 1
	mov rsi, data
	mov rdx, 1
	int 0x80

	; writing new line...
	mov rax, 3
	mov rdi, 1
	mov rsi, newline
	mov rdx, 1
	int 0x80
	; exit
	mov rax, 0
	int 0x80

section .data
	msg db 'Hello, from userspace test! type something with 3 chars: '
	len equ $ - msg
	data db 0, 0, 0, 0xA

	msg2 db 0xA, 'you entered: '
	len2 equ $ - msg2

	msg3 db 'attempting to write to FD 0...', 0xA
	len3 equ $ - msg3

	msg4 db 'rax is '
	len4 equ $ - msg4

	newline db 0xA
