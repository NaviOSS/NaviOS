#include "GDT.h"
#define GDT_ENTRIES 3

uint8_t GDT[GDT_ENTRIES*8];
void encodeGDTEntry(uint8_t* target, GDTEntry entry) {
    // encodedGDTEntry:
    /*
        struct {
            uint16_t limit1;

            uint16_t base1;
            uint8_t base2;

            uint8_t accessByte;
            uint8_t limit2Flags; // flags at the last 4 bits
            uint8_t base3;
        };
    */
    
    target[0] = entry.limit & 0xFF;
    
    target[1] = (entry.limit >> 8) & 0xFF;

    target[2] = entry.base & 0xFF;
    target[3] = (entry.base >> 8) & 0xFF;
    target[4] = (entry.base >> 16) & 0xFF;

    target[5] = entry.access;

    target[6] = (entry.limit >> 16) & 0xFF;
    target[6] |= (entry.flags << 4);

    target[7] = (entry.base >> 24) & 0xFF;
}

void initGDT() {
    GDTEntry null = {0, 0, 0, 0};
    encodeGDTEntry(&GDT[0], null);
}