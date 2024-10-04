#ifndef __nlibc__SRC_EXTRA_
#define __nlibc__SRC_EXTRA_

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <sys/types.h>

uint8_t *reverse(uint8_t *arg0, size_t arg1);
int itoa(size_t arg0, uint8_t *arg1, uint8_t arg2);

#endif