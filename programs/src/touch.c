#include <stdint.h>
#include <stdio.h>
#include <utils.h>
int main(size_t argc, Str **argv) {
  if (argc < 2) {
    printf("not enough arguments expected the filename");
    return -1;
  }

  Str *file_name = argv[1];

  int64_t attempt = create(file_name->data, file_name->len);
  if (attempt < 0) {
    printf("failed to touch `%S`, err: %d\n", file_name, attempt);
    return -2;
  }
  return 0;
}
