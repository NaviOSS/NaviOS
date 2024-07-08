#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

// VGA
typedef enum VGA_COLOR {
    VGA_COLOR_BLACK = 0,
    VGA_COLOR_RED = 4,
    VGA_COLOR_YELLOW = 14,
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
size_t terminal_row = 0;
size_t terminal_col = 0;

size_t strlen(const char* str) {
    size_t len = 0;
    while (str[len])
        len++;
    return len;
}

void initTerminal() {
    terminal_buffer = (uint16_t*) 0xB8000;
    
    for (size_t y = 0; y < VGA_HEIGHT; y++) {
        for (size_t x = 0; x < VGA_WIDTH; x++) {
            const size_t index = y * VGA_WIDTH + x;
            terminal_buffer[index] = vga_entry(' ', vga_entry_color(VGA_COLOR_WHITE, VGA_COLOR_BLACK));
        }
    }
}

void terminalPut(char* str, uint8_t color) {
    size_t len = strlen(str);
    for (size_t i = 0; i < len; i++) {
        if (str[i] == '\n') {
            terminal_row++;
            terminal_col = 0;
            continue;
        }
        terminal_buffer[terminal_col+terminal_row*VGA_WIDTH] = vga_entry(str[i], color);
        terminal_col++;
    }
}

void write(char* str) {
    terminalPut(str, vga_entry_color(VGA_COLOR_WHITE, VGA_COLOR_BLACK));
}

void kerr(char* err) {
    terminalPut(err, vga_entry_color(VGA_COLOR_RED, VGA_COLOR_BLACK));
}

void kwarn(char* warn) {
    terminalPut(warn, vga_entry_color(VGA_COLOR_YELLOW, VGA_COLOR_BLACK));
}
// KERNEL

void kernel_main() {
    initTerminal();

    write("Hello, world!\n");
    write("some more text");
    write(", and more...\n");
    kerr("NO MORE INFO\n");
    kwarn("WARNING\n");
}