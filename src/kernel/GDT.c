#include "GDT.h"
#include "kernel.h"

typedef enum FLAG {
    LONG = 1 << 5,
    IS_32BIT = 1 << 6,
    PAGE_LIMIT = 1 << 7,
} FLAG;

typedef enum ACCESS {
    ACCESSED = 1 << 0,
    READ = 1 << 1,
    WRITE = 1 << 1,
    DIR_DOWN = 1 << 2,
    EXECUTABLE = 1 << 3,
    NON_SYSTEM = 1 << 4,
    RING1 = 1 << 5,
    RING2 = 1 << 6,
    RING3 = RING1 | RING2,
    VALID = 1 << 7,
} ACCESS;

#define GDT_ENTRIES 3

uint8_t GDT[GDT_ENTRIES * 8];
GDTDescriptor GDTDesc = {.limit = sizeof(GDT) - 1, .base = (uint32_t)&GDT};

void encodeGDTEntry(uint8_t* target, GDTEntry entry) {
    target[0] = entry.limit & 0xFF;
    target[1] = (entry.limit >> 8) & 0xFF;

    target[2] = entry.base & 0xFF;
    target[3] = (entry.base >> 8) & 0xFF;
    target[4] = (entry.base >> 16) & 0xFF;

    target[5] = entry.access;

    target[6] = (entry.limit >> 16) & 0x0F;
    target[6] |= (entry.flags & 0xF0);

    target[7] = (entry.base >> 24) & 0xFF;
}

void setGDT() {
    asm volatile("lgdt %0" : : "m"(GDTDesc));
    // Reload segment registers
    asm volatile(
        "mov $0x10, %%ax \n\t"
        "mov %%ax, %%ds \n\t"
        "mov %%ax, %%es \n\t"
        "mov %%ax, %%fs \n\t"
        "mov %%ax, %%gs \n\t"
        "mov %%ax, %%ss \n\t"
        // Reload cs using far jump
        "push $0x08 \n\t"
        "push $1f \n\t"
        "retf \n\t"
        "1:\n\t"
        : : : "ax"
    );
}

void printSegmentRegisters() {
    uint16_t cs, ds, es, ss, fs, gs;
    asm volatile (
        "mov %%cs, %0 \n\t"
        "mov %%ds, %1 \n\t"
        "mov %%es, %2 \n\t"
        "mov %%ss, %3 \n\t"
        "mov %%fs, %4 \n\t"
        "mov %%gs, %5 \n\t"
        : "=r" (cs), "=r" (ds), "=r" (es), "=r" (ss), "=r" (fs), "=r" (gs)
    );

    write("CS: "); write_hex(cs); write("\n");
    write("DS: "); write_hex(ds); write("\n");
    write("ES: "); write_hex(es); write("\n");
    write("SS: "); write_hex(ss); write("\n");
    write("FS: "); write_hex(fs); write("\n");
    write("GS: "); write_hex(gs); write("\n");
}


void initGDT() {
    GDTEntry null = {0, 0, 0, 0};
    GDTEntry KernelCodeSeg = {.base = 0, .limit = 0xFFFFF, .access = VALID | READ | NON_SYSTEM | EXECUTABLE, .flags = IS_32BIT | PAGE_LIMIT};
    GDTEntry KernelDataSeg = {.base = 0, .limit = 0xFFFFF, .access = VALID | WRITE | NON_SYSTEM, .flags = IS_32BIT | PAGE_LIMIT};

    encodeGDTEntry(&GDT[0], null);
    encodeGDTEntry(&GDT[8], KernelCodeSeg);
    encodeGDTEntry(&GDT[16], KernelDataSeg);

    setGDT();
}
