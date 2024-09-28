#include "lexer.h"
#include <stdio.h>
#include <utils.h>

int main() {
  for (;;) {
    printf(">> ");
    Str input = readln();
    printf("%S\n", &input);
    Tokenizer tokenizer = __tokenizer__new__(input);

    while (!is_eof(&tokenizer)) {
      Token tok = next(&tokenizer);
      printf("{ lexeme: %S, type: %d, line: %d, col: %d }\n", &tok.lexeme,
             tok.type, tok.line, tok.col);
    }
    __str__destroy__(input);
  }

  return 0;
}
