#include "sys.h"
#include "mem.h"
#include <stdint.h>

void exit() { syscall(0, 0, 0, 0, 0); }
void yield() { syscall(1, 0, 0, 0, 0); }
void wait(int64_t pid) { syscall(11, pid, 0, 0, 0); }

int64_t chdir(uint8_t *ptr, size_t len) {
  return syscall(14, (uintptr_t)ptr, len, 0, 0);
}

int64_t getcwd(uint8_t *ptr, size_t len) {
  return syscall(15, (uintptr_t)ptr, len, 0, 0);
}

void *sbrk(ssize_t amount) { return (void *)syscall(18, amount, 0, 0, 0); }

void __stdlib__init__() { __malloc__init__(); }
