#include <stdint.h>
#include <stdio.h>
#include <sys.h>

int main() {
  int64_t fd = open_n(".");
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
