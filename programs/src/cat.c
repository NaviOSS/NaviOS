#include <stdint.h>
#include <stdio.h>
#include <sys/io.h>
#include <sys/raw.h>

int main(size_t argc, OsStr **argv) {
  if (argc < 2) {
    printf("not enough arguments expected the filename to cat!\n");
    return -1;
  }

  OsStr *filename = argv[1];
  ssize_t fd = open(filename->data, filename->len);
  if (fd < 0) {
    printf("failed opening `%S`, err: %d\n", filename, fd);
    return -2;
  }

  const DirEntry *entry = fstat(fd);
  if (entry == NULL) {
    printf("failed getting direntry of `%S`\n", filename);
    return -3;
  }

  uint8_t buffer[entry->size];
  int err = read(fd, buffer, entry->size);

  if (err < 0) {
    printf("failed reading file `%S`\n", filename);
    return -4;
  }

  // cat!
  printf("%.*s\n", entry->size, buffer);

  close(fd);
  return 0;
}
