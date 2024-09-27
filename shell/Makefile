STDLIB_PATH = ../stdlib

STDLIB = ${STDLIB_PATH}/libstd.a

CC = gcc
CFLAGS = -static -nostdlib -ffreestanding -I${STDLIB_PATH}/include -Iinclude

SRC_DIR = src
OBJ_DIR = obj
SRC_FILES = $(wildcard $(SRC_DIR)/*.c)
OBJ_FILES = $(patsubst $(SRC_DIR)/%.c, ${OBJ_DIR}/%.o, $(SRC_FILES))
BIN = nash

all: ${STDLIB} $(BIN)
$(BIN): $(STDLIB) $(OBJ_FILES)
	$(CC) $(CFLAGS) -fno-stack-protector $(OBJ_FILES) -Wl,-u _start -L${STDLIB_PATH} -lstd -o $@

${STDLIB}: ${STDLIB_PATH}
	cd ${STDLIB_PATH} && make

${OBJ_DIR}/%.o: ${SRC_DIR}/%.c
	@mkdir -p ${OBJ_DIR}
	${CC} ${CFLAGS} -c $< -o $@

clean:
	rm ${OBJ_FILES}
	rm -rf ${OBJ_DIR}
