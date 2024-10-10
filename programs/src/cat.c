#include <stdint.h>
#include <stdio.h>
#include <sys/io.h>
#include <sys/raw.h>

int main(size_t argc, char **argv) {
  if (argc < 2) {
    printf("not enough arguments expected the filename to cat!\n");
    return -1;
  }

  char *filename = argv[1];
  FILE *fd = fopen(filename, "r");
  if (fd == NULL) {
    printf("failed opening `%S`, err: %d\n", filename, fd);
    return -2;
  }

  DirEntry *entry = fstat(fd->fd);
  if (entry == NULL) {
    printf("failed getting direntry of `%S`\n", filename);
    return -3;
  }

  char buffer[entry->size];
  char *read = fgets(buffer, entry->size, fd);

  if (read == NULL) {
    printf("failed reading file `%S`\n", filename);
    return -4;
  }

  // cat!
  printf("%.*s\n", entry->size, read);

  fclose(fd);
  return 0;
}
