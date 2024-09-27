#include <stdio.h>
#include <utils.h>

int main(size_t argc, OsStr **argv) {
  if (argc == 1)
    printf("need at least one argument to echo!\n");

  for (int i = 1; i < argc; i++)
    printf("%S", argv[i]);

  printf("\n");

  return 0;
}
