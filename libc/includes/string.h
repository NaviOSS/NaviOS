#ifndef __nlibc__SRC_STRING_
#define __nlibc__SRC_STRING_

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <sys/types.h>

size_t strlen(const char *arg0);
const char *strerror(uint32_t arg0);
size_t strerrorlen_s(uint32_t arg0);
void *memset(void *arg0, int arg1, size_t arg2);
void *memcpy(void *arg0, const void *arg1, size_t arg2);

#endif