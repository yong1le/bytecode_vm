use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Star,
    Slash,
    Semicolon,
    Plus,
    Minus,
    Dot,
    Comma,
    Error
}

#[derive(Debug, Clone)]
pub struct Token {
  pub token_type: TokenType,
  pub token_str: String,
  pub lexeme: String,
  pub line: usize
}

impl fmt::Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self.token_type {
        TokenType::Error => write!(f, "[line {}] Error: Unexpected character: {}", self.line, self.lexeme),
        _ => write!(f, "{} {} null", self.token_str, self.lexeme)
    }
  }
}

pub struct Scanner {}
