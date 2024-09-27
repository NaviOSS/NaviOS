CC = gcc

CFLAGS = -Iinclude -Wall -Wextra -static -ffreestanding -nostdlib -fno-stack-protector

ASM = nasm
ASM_FLAGS = -f elf64
LIBRARY = libstd.a

SRC_DIR = src
OBJ_DIR = obj

SRC_FILES = $(wildcard $(SRC_DIR)/*.c)
SRC_ASM_FILES = $(wildcard $(SRC_DIR)/*.s)

OBJ_C_FILES = $(patsubst $(SRC_DIR)/%.c, $(OBJ_DIR)/%.o, $(SRC_FILES))
OBJ_ASM_FILES =$(patsubst $(SRC_DIR)/%.s, $(OBJ_DIR)/%.o, $(SRC_ASM_FILES))
OBJ_FILES = $(OBJ_C_FILES) $(OBJ_ASM_FILES)

all: $(LIBRARY)

$(LIBRARY): $(OBJ_FILES)
	ar rcs $@ $^
$(OBJ_DIR)/%.o: $(SRC_DIR)/%.c
	@mkdir -p $(OBJ_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

$(OBJ_DIR)/%.o: $(SRC_DIR)/%.s
	@mkdir -p $(OBJ_DIR)
	$(ASM) $(ASM_FLAGS) $< -o $@

clean:
	rm -f $(OBJ_DIR)/*.o $(LIBRARY)
.PHONY: all clean
