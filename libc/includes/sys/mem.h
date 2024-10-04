#ifndef __nlibc__SRC_SYS_MEM_
#define __nlibc__SRC_SYS_MEM_

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <sys/types.h>

void *sbrk(ssize_t arg0);

#endif