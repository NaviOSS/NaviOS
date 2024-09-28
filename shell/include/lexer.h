#pragma once
#include <stdint.h>
#include <utils.h>
#include <stdbool.h>

typedef enum TokenType: uint8_t {
  Normal,
  EOF,
} TokenType;

typedef struct Token {
  TokenType type;
  Str lexeme;
  uint32_t line, col;
} Token;

typedef struct Tokenizer {
  uint32_t line, col;
  size_t pos;
  const Str code;
} Tokenizer;

/// advances `self` and returns a Token that lives as long as `self.code`
Token next(Tokenizer *self);
/// creates a new Tokenizer
static inline Tokenizer __tokenizer__new__(const Str code) {
  Tokenizer tokenizer = { .line = 0,.col = 0, .pos = 0, .code = code };
  return tokenizer;
}
static inline bool is_eof(Tokenizer *self) {
  return self->pos >= self->code.len;
}
