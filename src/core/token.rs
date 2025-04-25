use std::fmt::{self};

use super::literal::Literal;

/// Enum to represent the different types of tokens in the language.
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
    /// The literal value stored by the token.
    pub literal: Literal,
    /// The actual string representation of the token.
    pub lexeme: String,
    /// The line number where the token was found.
    pub line: u32,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TokenType::LeftParen => "LEFT_PAREN",
                TokenType::RightParen => "RIGHT_PAREN",
                TokenType::LeftBrace => "LEFT_BRACE",
                TokenType::RightBrace => "RIGHT_BRACE",
                TokenType::Star => "STAR",
                TokenType::Slash => "SLASH",
                TokenType::Semicolon => "SEMICOLON",
                TokenType::Plus => "PLUS",
                TokenType::Minus => "MINUS",
                TokenType::Dot => "DOT",
                TokenType::Comma => "COMMA",
                TokenType::Equal => "EQUAL",
                TokenType::EqualEqual => "EQUAL_EQUAL",
                TokenType::BangEqual => "BANG_EQUAL",
                TokenType::Bang => "BANG",
                TokenType::LessThan => "LESS",
                TokenType::GreaterThan => "GREATER",
                TokenType::LessEqual => "LESS_EQUAL",
                TokenType::GreaterEqual => "GREATER_EQUAL",
                TokenType::String => "STRING",
                TokenType::Number => "NUMBER",
                TokenType::Identifier => "IDENTIFIER",
                TokenType::And => "AND",
                TokenType::Class => "CLASS",
                TokenType::Else => "ELSE",
                TokenType::False => "FALSE",
                TokenType::For => "FOR",
                TokenType::Fun => "FUN",
                TokenType::If => "IF",
                TokenType::Nil => "NIL",
                TokenType::Or => "OR",
                TokenType::Print => "PRINT",
                TokenType::Return => "RETURN",
                TokenType::Super => "SUPER",
                TokenType::This => "THIS",
                TokenType::True => "TRUE",
                TokenType::Var => "VAR",
                TokenType::While => "WHILE",
                TokenType::Eof => "EOF",
            }
        )
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.literal {
            Literal::Nil => write!(f, "{} {} null", self.token, self.lexeme),
            _ => write!(f, "{} {} {}", self.token, self.lexeme, self.literal),
        }
    }
}
