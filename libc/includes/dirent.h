#ifndef __nlibc__SRC_DIRENT_
#define __nlibc__SRC_DIRENT_

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <sys/types.h>

#include "sys/raw.h"
typedef struct DIR {
  size_t current_index;
  ssize_t ri;
  ssize_t dir_ri;
} DIR;

DIR *opendir(const char *arg0);
DirEntry *readdir(DIR *arg0);
int telldir(DIR *arg0);
int closedir(DIR *arg0);

#endif