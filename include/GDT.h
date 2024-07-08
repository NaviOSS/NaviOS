#include <stdint.h>
#include "kernel.h"

typedef struct {
    uint32_t base;
    uint32_t limit;
    uint8_t access;
    uint8_t flags;
} GDTEntry;

typedef struct {
    uint16_t limit;
    uint32_t base;
} __attribute__((packed)) GDTDescriptor;

void encodeGDTEntry(uint8_t* target, GDTEntry entry);
void initGDT();
void printSegmentRegisters();