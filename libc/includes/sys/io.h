#ifndef __nlibc__SRC_SYS_IO_
#define __nlibc__SRC_SYS_IO_

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <sys/types.h>

#include "raw.h"
ssize_t open(const uint8_t *arg0, size_t arg1);
ssize_t close(ssize_t arg0);
ssize_t diriter_open(ssize_t arg0);
ssize_t diriter_close(ssize_t arg0);
DirEntry *diriter_next(ssize_t arg0);
DirEntry *fstat(ssize_t arg0);
ssize_t read(ssize_t arg0, uint8_t *arg1, size_t arg2);
ssize_t write(ssize_t arg0, const uint8_t *arg1, size_t arg2);

#endif