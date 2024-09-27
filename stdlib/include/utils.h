#include <stddef.h>
#include <stdint.h>

/// wries the textual version of `val` into ptr
/// returns the start of the written text it will write from backwards
/// or returns -1 if fail
/// str
char* itoa(uint64_t val, char* str, int base);

/// reverses a str
char* reverse(char* str, size_t length);

/// the layout of NaviOS strings
/// rust &str like
typedef struct Str {
  size_t len;
  uint8_t data[];
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
