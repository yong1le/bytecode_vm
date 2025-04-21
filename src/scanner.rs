use std::fmt;
use std::io::{self, Write};

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
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub token_str: String,
    pub lexeme: String,
    pub line: u32,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} null", self.token_str, self.lexeme)
    }
}

pub struct Scanner<'a> {
    source: &'a String,
    tokens: Vec<Token>,
}

impl Scanner<'_> {
    pub fn new(source: &String) -> Scanner {
        return Scanner {
            source,
            tokens: vec![],
        };
    }

    fn add_token(&mut self, token_type: TokenType, token_str: &str, lexeme: &str, line: u32) {
        self.tokens.push(Token {
            token_type,
            token_str: token_str.to_string(),
            lexeme: lexeme.to_string(),
            line,
        });
    }

    pub fn print(&self) {
        self.tokens.iter().for_each(|t| println!("{t}"));
        println!("EOF  null")
    }

    pub fn tokenize(&mut self) -> Result<(), ()> {
        let mut has_error = false;
        let mut chars = self.source.chars().peekable();
        let mut line = 1u32;

        while let Some(c) = chars.next() {
            match c {
                '(' => self.add_token(TokenType::LeftParen, "LEFT_PAREN", "(", line),
                ')' => self.add_token(TokenType::RightParen, "RIGHT_PAREN", ")", line),
                '{' => self.add_token(TokenType::LeftBrace, "LEFT_BRACE", "{", line),
                '}' => self.add_token(TokenType::RightBrace, "RIGHT_BRACE", "}", line),
                '*' => self.add_token(TokenType::Star, "STAR", "*", line),
                ';' => self.add_token(TokenType::Semicolon, "SEMICOLON", ";", line),
                '+' => self.add_token(TokenType::Plus, "PLUS", "+", line),
                '-' => self.add_token(TokenType::Minus, "MINUS", "-", line),
                '.' => self.add_token(TokenType::Dot, "DOT", ".", line),
                ',' => self.add_token(TokenType::Comma, "COMMA", ",", line),
                '=' => {
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        self.add_token(TokenType::EqualEqual, "EQUAL_EQUAL", "==", line)
                    } else {
                        self.add_token(TokenType::Equal, "EQUAL", "=", line)
                    }
                }
                '!' => {
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        self.add_token(TokenType::BangEqual, "BANG_EQUAL", "!=", line)
                    } else {
                        self.add_token(TokenType::Bang, "BANG", "!", line)
                    }
                }
                '<' => {
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        self.add_token(TokenType::LessEqual, "LESS_EQUAL", "<=", line)
                    } else {
                        self.add_token(TokenType::LessThan, "LESS", "<", line)
                    }
                }
                '>' => {
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        self.add_token(TokenType::GreaterEqual, "GREATER_EQUAL", ">=", line)
                    } else {
                        self.add_token(TokenType::GreaterThan, "GREATER", ">", line)
                    }
                }
                '/' => {
                    if chars.peek() == Some(&'/') {
                        while chars.peek() != Some(&'\n') && chars.peek() != None{
                            chars.next();
                        }
                    } else {
                        self.add_token(TokenType::Slash, "SLASH", "/", line)
                    }
                }
                ' ' | '\t' | '\r' => (),
                '\n' => {
                    line += 1;
                }
                _ => {
                    has_error = true;
                    writeln!(
                        io::stderr(),
                        "[line {}] Error: Unexpected character: {}",
                        line,
                        c
                    )
                    .unwrap();
                }
            };
        }
        if has_error {
            return Err(());
        } else {
            Ok(())
        }
    }
}
