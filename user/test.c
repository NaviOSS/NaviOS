//! compile with `gcc -no-pie -nostdlib -static -ffreestanding -fno-stack-protector user/test.c -o user/test` if you have gcc
// you just need to compile an ELF with type EXE ig
#include <stdint.h>
#include <stddef.h>

int64_t syscall(uint64_t num, uint64_t arg0, uint64_t arg1, uint64_t arg3) {
    int64_t result;
    asm volatile (
        "mov %1, %%rax\n" 
        "mov %2, %%rdi\n"  
        "mov %3, %%rsi\n"  
        "mov %4, %%rdx\n"  
        "int $0x80\n"      
        "mov %%rax, %0\n"  
        : "=r" (result)    
        : "r" (num), "r" (arg0), "r" (arg1), "r" (arg3)  
        : "rax", "rdi", "rsi", "rdx"  
    );
    return result;
}

size_t strlen(const char* str) {
	size_t len = 0;
	const char* ch = str;

	while (*ch != 0) {
		ch += 1;
		len += 1;
	}

	return len;
}

int64_t write(uint32_t fd, const void* ptr, size_t len) {
	return syscall(3, fd, (uint64_t) ptr, len);
}

int64_t read(uint32_t fd, void* ptr, size_t len) {
	return syscall(4, fd, (uint64_t) ptr, len);
}

void writeout(const char* ptr, size_t len) {
	write(1, ptr, len);
}

void readin(char* ptr, size_t len) {
	read(0, ptr, len);
}

void print(const char* str) {
	size_t len = strlen(str);
	writeout(str, len);
}

void pexit() {
	syscall(0, 0, 0 ,0);
}

int _start() {
	char data[] = {0, 0, 0,	'\n'};

	print("Hello, from userspace test! type something with 3 chars: ");
	readin(data, 3);

	print("\nyou entered: ");
	writeout(data, 4);

	print("attempting to write to FD 0...\n");
	
	int64_t rax = write(0, data, 3);
	print("rax is ");
	data[0] = (-rax) + '0';

	writeout(data, 1);
	print("\n");

	pexit();
	return 0;
}
