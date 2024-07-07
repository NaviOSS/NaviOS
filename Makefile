TARGET=i686-elf
CC=${TARGET}-gcc
LD=${TARGET}-ld

CFLAGS=-Wall -O2 -nostdlib -nostartfiles -ffreestanding -mgeneral-regs-only -Iinclude
BUILD = build
SRC = src

all: kernel.img

$(BUILD)/%_c.o: $(SRC)/%.c
	mkdir -p ${@D}
	$(CC) $(CFLAGS) -c $< -o $@

$(BUILD)/%_s.o: $(SRC)/%.S 
	mkdir -p ${@D}
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -rf $(BUILD) *.o *.elf *.img

C_FILES = $(wildcard $(SRC)/*.c)
ASM_FILES = $(wildcard $(SRC)/*.S)

OBJ_FILES = $(C_FILES:$(SRC)/%.c=$(BUILD)/%_c.o)
OBJ_FILES += $(ASM_FILES:$(SRC)/%.S=$(BUILD)/%_s.o)

DEP_FILES = $(OBJ_FILES:%.o=%.d)
-include $(DEP_FILES)

kernel.img: $(SRC)/linker.ld $(OBJ_FILES)
	$(LD) -T $(SRC)/linker.ld -o kernel.img  $(OBJ_FILES)