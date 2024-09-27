#include "utils.h"
#include <stdio.h>

int main() {
  for (;;) {
    printf(">> ");
    Str input = readln();
    printf("%S\n", &input);
    __str__destroy__(input);
  }

  return 0;
}
