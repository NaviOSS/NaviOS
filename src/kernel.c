#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

// VGA
typedef enum VGA_COLOR {
    VGA_COLOR_BLACK = 0,
    VGA_COLOR_WHITE = 2,
} VGA_COLOR;

static const size_t VGA_WIDTH = 80;
static const size_t VGA_HEIGHT = 25;

uint8_t vga_entry_color(VGA_COLOR fg, VGA_COLOR bg) 
{
	return fg | bg << 4;
}
// TERMINAL

// KERNEL
void kernel_main() {
    while (1) {
        ;
    }
}