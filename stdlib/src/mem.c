#include "mem.h"
#include "sys.h"
#include <stdint.h>
#include <stdio.h>

Chunk *head;
#define INIT_SIZE 4096
/// increases heap size and adds a free Chunk with size `size` at the end
Chunk *add_free(size_t size) {
  Chunk *ptr = sbrk(0);
  void *end = sbrk(size + sizeof(Chunk));
  if (end == NULL)
    return NULL;

  ptr->size = size;
  ptr->free = true;
  return ptr;
}
void __malloc__init__() { head = add_free(INIT_SIZE); }

/// finds a free chunk starting from `head`
Chunk *find_free(size_t size) {
  Chunk *current = head;
  void *end = sbrk(0);

  while ((void *)current < end) {
    if (current->size >= size)
      return current;
    current = (Chunk *)((size_t)current + current->size + sizeof(Chunk));
  }

  return NULL;
}

void *malloc(size_t size) {
  size_t asize = align_up(size, MALLOC_SIZE_ALIGN);
  Chunk *block = find_free(size);
  // attempt to increase heap size
  if (block == NULL) {
    block = add_free(asize);
    if (block == NULL)
      return NULL;
  }

  // divide block
  if (block->size > asize) {
    size_t diff = block->size - asize;
    if (diff >= sizeof(Chunk) + MALLOC_SIZE_ALIGN) {
      Chunk *new_chunk = (Chunk *)(block->data + block->size - diff);
      new_chunk->free = true;
      new_chunk->size = diff - sizeof(Chunk);

      block->size -= diff;
    }
  }

  block->free = false;
  return block->data;
}

void free(void *ptr) {
  if (ptr == NULL)
    return;

  Chunk *chunk = ptr - sizeof(Chunk);
  chunk->free = true;

  // give the chunk back to the os if it is at the end
  if (((size_t)chunk + chunk->size) == (size_t)sbrk(0)) {
    sbrk(-(chunk->size + sizeof(Chunk)));
    return;
  }
}
