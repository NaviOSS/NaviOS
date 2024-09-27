#include <stdint.h>
#include <stddef.h>
#include <sys/types.h>
#pragma once
__attribute__((always_inline))
static inline int64_t syscall(uint64_t num, uint64_t arg0, uint64_t arg1, uint64_t arg3, uint64_t arg4) {
    int64_t result;
    __asm__ volatile (
        "mov %1, %%rax\n" 
        "mov %2, %%rdi\n"  
        "mov %3, %%rsi\n"  
        "mov %4, %%rdx\n"          
	"mov %5, %%rcx\n"  
        "int $0x80\n"      
        "mov %%rax, %0\n"  
        : "=r" (result)    
        : "r" (num), "r" (arg0), "r" (arg1), "r" (arg3), "r" (arg4)
        : "rax", "rdi", "rsi", "rdx"  
    );
    return result;
}

void exit();
void yield();
void wait(int64_t pid);

/// changes the cwd to the utf-16 encoded string at ptr of len bytes
int64_t chdir(uint8_t *ptr, size_t len);
/// gets the current work dir
/// returns -1 if len is smaller then the cwd len returns the cwd len on success
int64_t getcwd(uint8_t *ptr, size_t len);
/// extends program break by ammount
/// on fail returns null
void* sbrk(ssize_t amount);
