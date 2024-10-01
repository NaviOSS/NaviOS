#ifndef __nlibc__SRC_SYS_IO_
#define __nlibc__SRC_SYS_IO_

#include <stddef.h>
#include <stdint.h>
#include <unistd.h>
#include <stdbool.h>

ssize_t open(const uint8_t *arg0, size_t arg1);
ssize_t close(ssize_t arg0);

#endif