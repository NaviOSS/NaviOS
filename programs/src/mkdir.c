#include <stdint.h>
#include <stdio.h>
#include <utils.h>

int main(size_t argc, Str **argv) {
  if (argc < 2) {
    printf("not enough arguments expected directory name\n");
    return -1;
  }

  Str *directory_name = argv[1];
  int64_t attempt = createdir(directory_name->data, directory_name->len);
  if (attempt < 0) {
    printf("failed to create directory `%S`, err: %d\n", directory_name,
           attempt);
    return -2;
  }

  return 0;
}
