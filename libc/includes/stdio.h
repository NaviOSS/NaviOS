#ifndef __nlibc__SRC_STDIO_
#define __nlibc__SRC_STDIO_

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <sys/types.h>

typedef struct FILE {
  ssize_t fd;
  uint8_t mode;
} FILE;

extern FILE stdin;
extern FILE stdout;
FILE *fopen(const char *arg0, const char *arg1);
int fclose(FILE *arg0);
int fgetc(FILE *arg0);
int getc(FILE *arg0);
char *fgets(char *arg0, int arg1, FILE *arg2);
char *gets_s(char *arg0, size_t arg1);
int getchar();
char *fgetline(FILE *arg0, size_t *arg1);
int zprintf(const uint8_t *arg0, ...);
int printf(const char *arg0, ...);

#endif