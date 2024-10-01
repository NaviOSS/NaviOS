#ifndef __nlibc__SRC_STRING_
#define __nlibc__SRC_STRING_

#include <stddef.h>
#include <stdint.h>

size_t strlen(const uint8_t *arg0);
const uint8_t *strerror(uint32_t arg0);
size_t strerrorlen_s(uint32_t arg0);

#endif