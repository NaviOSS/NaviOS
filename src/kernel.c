#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

// VGA
typedef enum VGA_COLOR {
    VGA_COLOR_BLACK = 0,
    VGA_COLOR_WHITE = 15,
} VGA_COLOR;

static const size_t VGA_WIDTH = 80;
static const size_t VGA_HEIGHT = 25;

uint8_t vga_entry_color(VGA_COLOR fg, VGA_COLOR bg) 
{
	return fg | bg << 4;
}

static inline uint16_t vga_entry(unsigned char uc, uint8_t color) 
{
	return (uint16_t) uc | (uint16_t) color << 8;
}

// TERMINAL
uint16_t* terminal_buffer;

// KERNEL
void kernel_main() {
	terminal_buffer = (uint16_t*) 0xB8000;
    terminal_buffer[0] = vga_entry('H', vga_entry_color(VGA_COLOR_WHITE, VGA_COLOR_BLACK));
}