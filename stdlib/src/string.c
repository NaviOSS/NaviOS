#include "string.h"
size_t strlen(const char *str) {
  size_t len = 0;
  const char *ch = str;

  while (*ch != 0) {
    ch++;
    len++;
  }

  return len;
}

int strcmp(const char *str1, const char *str2) {
  const char *p1 = str1;
  const char *p2 = str2;

  while (*p1 != 0) {
    if (*p2 == 0 || *p1 > *p2)
      return 1;

    if (*p1 < *p2)
      return -1;

    p1++;
    p2++;
  }

  if (*p2 != 0)
    return 1;

  return 0;
}

char* strcat(char* dest, const char*src){
  for(; *dest != 0; ++dest);

  while(*src != 0){
    *dest = *src;
    dest++;
    src++;
  }

  *dest = 0;

  return dest;
}

char* strcpy(char *dest, const char *src){
  char *save = dest;

  while(*src != 0){
    *dest = *src;
    dest++;
    src++;
  }

  *dest = 0;

  return save;
}