#include <sys.h>

int main() { return 0; }

void _start() {
  main();
  pexit();
}
