#include "lexer.h"
#include <stdbool.h>
#include <stdint.h>
#include <string.h>

static inline char at(Tokenizer *self) {
  if (self->pos >= self->code.len)
    return '\0';

  return self->code.data[self->pos];
}
static inline char eat(Tokenizer *self) {
  char prev = at(self);
  self->pos += 1;

  return prev;
}

static inline bool is_skippable(char x) {
  return x == ' ' || x == '\n' || x == '\t';
}
static inline bool is_eof(Tokenizer *self) {
  return self->pos >= self->code.len;
}

static inline Token make_token_with(Tokenizer *self, TokenType type,
                                    const char *lexeme) {
  size_t len = strlen(lexeme);
  uint8_t *start_ptr = (uint8_t *)lexeme;

  Str lexeme_slice = {.len = len, .data = start_ptr};
  Token token = {.type = type,
                 .lexeme = lexeme_slice,
                 .line = self->line,
                 .col = self->col};

  return token;
}
static inline Token make_token(Tokenizer *self, TokenType type, size_t start) {
  size_t len = self->pos - start - 1;
  uint8_t *start_ptr = self->code.data + start;

  Str lexeme = {.len = len, .data = start_ptr};
  Token token = {
      .type = type, .lexeme = lexeme, .line = self->line, .col = self->col};

  return token;
}

Token next(Tokenizer *self) {
  if (is_eof(self)) {
    return make_token_with(self, EOF, "<EOF>");
  }

  while (is_skippable(at(self))) {
    switch (at(self)) {
    case '0':
    case '\t':
      self->col += 1;
      self->pos += 1;
      break;
    case '\n':
      self->col = 0;
      self->line += 1;
      self->pos += 1;
      break;
    default:
      break;
    }
  }

  switch (at(self)) {
  default: {
    size_t start = self->pos;
    while (!is_skippable(at(self)) && !is_eof(self)) {
      eat(self);
    }
    return make_token(self, Normal, start);
  }
  }
}
