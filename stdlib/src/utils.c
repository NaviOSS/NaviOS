#include "utils.h"
#include "sys.h"
#include <stdint.h>
char *reverse(char *str, size_t length) {
  int start = 0;
  int end = length - 1;
  while (start < end) {
    char tmp = str[start];
    str[start] = str[end];
    str[end] = tmp;

    start++;
    end--;
  }

  return str;
}

char *itoa(uint64_t val, char *str, int base) {
  if (base < 2 || base > 34)
    return (char *)-1;

  int i = 0;

  if (val == 0) {
    str[i++] = '0';
    str[i] = '\0';
    return str;
  }

  while (val != 0) {
    int rem = val % base;
    str[i++] = (rem > 9) ? (rem - 10) + 'a' : rem + '0';
    val /= base;
  }

  str[i] = '\0';
  return reverse(str, i);
}

int64_t sysinfo(SysInfo *info) { return syscall(16, (uintptr_t)info, 0, 0, 0); }

int64_t pcollect(ProcessInfo info[], size_t len) {
  return syscall(17, (uintptr_t)info, len, 0, 0);
}
