use std::fmt::{self};

/// Enum to represent the different types of tokens in the language.
#[derive(Debug, Clone, PartialEq, Copy)]
#[repr(u8)] // NOTE: This should be the default
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
    Equal,
    EqualEqual,
    BangEqual,
    Bang,
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    String,
    Number,
    Identifier,

    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

/// Struct to encapsolate all useful information about a token.
#[derive(Debug, Clone)]
pub struct Token {
    /// The type of the token.
    pub token: TokenType,
    /// The actual string representation of the token.
    pub lexeme: String,
    /// The line number where the token was found.
    pub line: u32,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} '{}'", self.token, self.lexeme)
    }
}
