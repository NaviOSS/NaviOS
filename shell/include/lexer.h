#pragma once
#include <stdint.h>
#include <utils.h>

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
