#pragma once
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

/// wries the textual version of `val` into ptr
/// returns the start of the written text it will write from backwards
/// or returns -1 if fail
/// str
char* itoa(uint64_t val, char* str, int base);

/// reverses a str
char* reverse(char* str, size_t length);

/// the layout of NaviOS strings
typedef struct OsStr {
  size_t len;
  uint8_t data[];
} OsStr;
// like `OsStr` but instead of data being right next to len, it is a pointer
// rust &str like
typedef struct Str {
  size_t len;
  uint8_t *data;
} Str;

typedef struct SysInfo {
  size_t mem_total;
  size_t mem_used;

  size_t processes_count;
} SysInfo;

int64_t sysinfo(SysInfo* info);

typedef struct ProcessInfo {
  uint64_t ppid;
  uint64_t pid;
  uint8_t name[64];
  uint8_t status;
} ProcessInfo;

int64_t pcollect(ProcessInfo info[], size_t len);

// some inline functions which peforms some operations on heap allocated `Str`
/// creates a new heap allocated str
static inline Str __str__new__() {
  Str str = { .len = 0, .data = malloc(1) };
  return str;
}
/// pushes `c` to str
static inline void __str_push__(Str *str, char c) {
  size_t len = str->len + 1;
  uint8_t* data = realloc(str->data, len);

  str->len = len;
  str->data = data;
  str->data[str->len - 1] = c;
}
/// destroies heap allocated str
static inline void __str__destroy__(Str str) {
  free(str.data);
}
