#include <stdio.h>
#include <sys/raw.h>

int main(size_t argc, OsStr **argv) {
  if (argc == 1) {
    printf("need at least one argument to echo!\n");
    return -1;
  }

  for (int i = 1; i < argc; i++)
    printf("%s", argv[i]->data);

  printf("\n");

  return 0;
}
