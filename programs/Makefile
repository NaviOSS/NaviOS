STDLIB_PATH = ../libc
INCLUDE_PATH = $(STDLIB_PATH)/includes
STDLIB_OUT = $(STDLIB_PATH)/zig-out/lib
STDLIB = ${STDLIB_PATH}/zig-out/lib/liblibc.a

CC = gcc
CFLAGS = -static -nostdlib -ffreestanding -I${INCLUDE_PATH} -fno-stack-protector

SRC_DIR = src
BUILD_DIR = build
OBJ_DIR = obj
SRC_FILES = $(wildcard $(SRC_DIR)/*.c)
EXE_FILES = $(patsubst $(SRC_DIR)/%.c, ${BUILD_DIR}/%, $(SRC_FILES))

all: ${STDLIB} ${EXE_FILES}

${STDLIB}: ${STDLIB_PATH}
	cd ${STDLIB_PATH} && zig build headergen

${BUILD_DIR}/%: ${SRC_DIR}/%.c
	@mkdir -p ${BUILD_DIR}
	${CC} ${CFLAGS} $< -Wl,-u _start -L${STDLIB_OUT} -llibc -o $@

clean:
	rm -rf ${LIB_DIR}
	rm -rf ${BUILD_DIR}
