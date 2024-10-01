#include "stdio.h"
#include "string.h"
#include "sys.h"
#include "utils.h"
#include <stdarg.h>
#include <stdint.h>
#include <sys/types.h>

ssize_t open(const void *path_ptr, size_t len) {
  size_t fd;
  int err = syscall(2, (uint64_t)path_ptr, len, (uintptr_t)&fd, 0);
  return err == 0 ? fd : -1;
}

ssize_t close(uint32_t fd) { return syscall(5, fd, 0, 0, 0) == 0 ? 0 : -1; }

int64_t write(uint32_t fd, const void *ptr, size_t len) {
  return syscall(3, fd, (uint64_t)ptr, len, 0);
}

int64_t read(uint32_t fd, void *ptr, size_t len) {
  return syscall(4, fd, (uint64_t)ptr, len, 0);
}

int64_t create(const void *path, size_t path_len) {
  return syscall(6, (uint64_t)path, path_len, 0, 0);
}

int64_t createdir(const void *path, size_t path_len) {
  return syscall(7, (uint64_t)path, path_len, 0, 0);
}

/// wrapper around open that takes a null-terminated path
int64_t open_n(const char *path) {
  size_t len = strlen(path);

  return open(path, len);
}

/// wrapper around create that takes a null terminated path and a null
/// terminated name
int64_t create_n(const char *path) {
  size_t path_len;

  path_len = strlen(path);

  return create(path, path_len);
}

int printf(const char *format, ...) {
  int i = 0;

  const char *current;
  va_list arg;
  va_start(arg, format);

  for (current = format; *current != '\0'; current++) {
    const char *start = current;
    int len = 0;

    while (*current != '%' && *current != '\0') {
      current++;
      len++;
    }

    write(1, start, len);
    if (*current == '\0')
      return 0;

    current++;
    switch (*current) {
    case 'd':
      i = va_arg(arg, int);
      if (i < 0) {
        i = -i;
        sputc('-');
      }

      char result[10];
      char *start = itoa(i, result, 10);

      printf("%s", start);
      break;
    case 's':
      printf(va_arg(arg, char *));
      break;
    case 'S': {
      Str *str = va_arg(arg, Str *);
      printf("%.*s", str->len, str->data);
      break;
    }
    case 'x': {
      uint64_t i = va_arg(arg, uint64_t);
      char result[17];

      char *start = itoa(i, result, 16);
      printf("0x%s", start);
      break;
    }
    case 'p': {
      void *ptr = va_arg(arg, void *);
      printf("%x", ptr);
      break;
    }
    case '.':
      current++;
      switch (*current) {
      case '*':
        current++;
        int num = va_arg(arg, int);
        switch (*current) {
        case 's':
          write(1, va_arg(arg, char *), num);
          break;
        }
        break;
      }
      break;
    }
  }

  va_end(arg);
  return 0;
}

char getchar() {
  char c = '\0';
  read(0, &c, 1);
  return c;
}

/// gets a str from stdin places it in ptr, reads until \n
/// doesn't include \n
/// returns the length of the str
/// if max is reached it will return -1 instead
int getstr(char *ptr, int max) {
  for (int i = 0; i < max; i++) {
    char c = getchar();
    if (c == '\n')
      return i;
    ptr[i] = c;
  }

  if (getchar() == '\n')
    return max;
  return -1;
}

int64_t diriter_open(int64_t ri) { return syscall(8, ri, 0, 0, 0); }

int64_t diriter_close(int64_t ri) { return syscall(9, ri, 0, 0, 0); }

int64_t diriter_next(int64_t ri, DirEntry *direntry) {
  return syscall(10, ri, (uintptr_t)direntry, 0, 0);
}

int64_t fstat(int64_t ri, DirEntry *direntry) {
  return syscall(12, ri, (uintptr_t)direntry, 0, 0);
}

Str readln() {
  Str str = __str__new__();
  char c;
  while ((c = getchar()) && c != '\n') {
    __str_push__(&str, c);
  }

  return str;
}
