#include "mem.h"
#include "sys.h"
#include <stdint.h>

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

/// combines free block starting from head
void anti_fragmentation() {
  Chunk *current = head;
  for (;;) {
    Chunk *next = (Chunk *)((size_t)current + current->size + sizeof(Chunk));
    if (next == sbrk(0))
      break;

    if (next->free && current->free)
      current->size += next->size + sizeof(Chunk);
    else if (!next->free)
      break;
    current = next;
  }
}

void *calloc(size_t size) {
  uint8_t *ptr = malloc(size);
  for (size_t i = 0; i < size; i++) {
    ptr[i] = 0;
  }

  return ptr;
}

void *realloc(void *ptr, size_t size) {
  if (size == 0) {
    free(ptr);
    return NULL;
  }
  Chunk *chunk = ptr - sizeof(Chunk);

  if (chunk->size < size) {
    // TODO: improve this so it combines with the next block?
    anti_fragmentation();

    void *new = malloc(size);
    memcpy(new, ptr, chunk->size);
    free(ptr);

    return new;
  }

  return ptr;
}

void free(void *ptr) {
  if (ptr == NULL)
    return;

  Chunk *chunk = ptr - sizeof(Chunk);
  chunk->free = true;

  // give the chunk back to the os if it is at the end
  if (((size_t)chunk + chunk->size) == (size_t)sbrk(0) && chunk != head) {
    sbrk(-(chunk->size + sizeof(Chunk)));
    return;
  }

  anti_fragmentation();
}

void memcpy(void *dest, const void *src, size_t size) {
  for (size_t i = 0; i < size; i++) {
    ((uint8_t *)dest)[i] = ((uint8_t *)src)[i];
  }
}
