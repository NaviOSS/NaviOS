#ifndef __nlibc__SRC_STDLIB_
#define __nlibc__SRC_STDLIB_

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <sys/types.h>

extern struct {
  size_t size;
  bool free;
  uint8_t data_off[8];
} *head;
void __malloc__init__();
void *malloc(size_t arg0);
void free(void *arg0);
void *realloc(void *arg0, size_t arg1);

#endif