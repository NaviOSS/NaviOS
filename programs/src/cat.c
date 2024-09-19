#include <stdint.h>
#include <stdio.h>
#include <utils.h>

int main(size_t argc, Str **argv) {
  if (argc < 2) {
    printf("not enough arguments expected the filename to cat!\n");
    return -1;
  }

  Str *filename = argv[1];
  int64_t fd = open(filename->data, filename->len);
  if (fd < 0) {
    printf("failed opening `%S`, err: %d\n", filename, fd);
    return -2;
  }

  DirEntry entry = {};
  int64_t attempt = fstat(fd, &entry);
  if (attempt < 0) {
    printf("failed getting direntry of `%S`\n", filename);
    return -3;
  }

  uint8_t buffer[entry.size];
  attempt = read(fd, buffer, entry.size);

  if (attempt < 0) {
    printf("failed reading file `%S`\n", filename);
    return -4;
  }

  // cat!
  printf("%.*s\n", entry.size, buffer);

  close(fd);
  return 0;
}
