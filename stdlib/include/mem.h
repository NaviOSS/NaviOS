#include <stddef.h>
#include <stdbool.h>
#include <stdint.h>
#pragma once

#define align_up(x, alignment) (x + alignment - 1) & ~(alignment - 1)
#define MALLOC_SIZE_ALIGN 16

typedef struct Chunk {
  size_t size;  
  bool free;
  uint8_t data[] __attribute__((aligned(8)));
} Chunk;

void* malloc(size_t size);
void free(void *ptr);
void* calloc(size_t size);
void* realloc(void *ptr, size_t size);

void __malloc__init__();

void memcpy(void *dest, const void *src, size_t size);
