STDLIB_PATH = ../stdlib

STDLIB = ${STDLIB_PATH}/libstd.a

CC = gcc
CFLAGS = -static -nostdlib -ffreestanding -I${STDLIB_PATH}/include -fno-stack-protector

SRC_DIR = src
BUILD_DIR = build
OBJ_DIR = obj
SRC_FILES = $(wildcard $(SRC_DIR)/*.c)
EXE_FILES = $(patsubst $(SRC_DIR)/%.c, ${BUILD_DIR}/%, $(SRC_FILES))

all: ${STDLIB} ${EXE_FILES}

${STDLIB}: ${STDLIB_PATH}
	cd ${STDLIB_PATH} && make

${BUILD_DIR}/%: ${SRC_DIR}/%.c
	@mkdir -p ${BUILD_DIR}
	${CC} ${CFLAGS} $< -Wl,-u _start -L${STDLIB_PATH} -lstd -o $@

clean:
	rm -rf ${LIB_DIR}
	rm ${EXE_FILES}
	rm -rf ${BUILD_DIR}
