#include <stdint.h>
#include <stdio.h>
#include <sys.h>

int main() {
  uint8_t buffer[1024];

  size_t len = getcwd(buffer, 1024);
  if (len < 0) {
    printf("ls: failed getting the current work dir!\n");
    return -1;
  }

  int64_t fd = open(buffer, len);
  if (fd < 0) {
    printf("ls: failed opening current work dir\n");
    return -1;
  }

  int64_t diriter = diriter_open(fd);
  if (fd < 0) {
    printf("ls: failed retriving items for the current work dir\n");
    return -2;
  }

  for (;;) {
    DirEntry entry = {};
    diriter_next(diriter, &entry);

    if (entry.name_length == 0)
      break;

    printf("%.*s\n", entry.name_length, entry.name);
  }

  diriter_close(diriter);
  close(fd);
  return 0;
}
