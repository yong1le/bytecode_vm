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
    String,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::LeftParen => write!(f, "LEFT_PAREN"),
            TokenType::RightParen => write!(f, "RIGHT_PAREN"),
            TokenType::LeftBrace => write!(f, "LEFT_BRACE"),
            TokenType::RightBrace => write!(f, "RIGHT_BRACE"),
            TokenType::Star => write!(f, "STAR"),
            TokenType::Slash => write!(f, "SLASH"),
            TokenType::Semicolon => write!(f, "SEMICOLON"),
            TokenType::Plus => write!(f, "PLUS"),
            TokenType::Minus => write!(f, "MINUS"),
            TokenType::Dot => write!(f, "DOT"),
            TokenType::Comma => write!(f, "COMMA"),
            TokenType::Equal => write!(f, "EQUAL"),
            TokenType::EqualEqual => write!(f, "EQUAL_EQUAL"),
            TokenType::BangEqual => write!(f, "BANG_EQUAL"),
            TokenType::Bang => write!(f, "BANG"),
            TokenType::LessThan => write!(f, "LESS"),
            TokenType::GreaterThan => write!(f, "GREATER"),
            TokenType::LessEqual => write!(f, "LESS_EQUAL"),
            TokenType::GreaterEqual => write!(f, "GREATER_EQUAL"),
            TokenType::String => write!(f, "STRING"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub literal: String,
    pub lexeme: String,
    pub line: u32,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.token_type, self.lexeme, self.literal)
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

    fn add_token(&mut self, token_type: TokenType, literal: &str, lexeme: &str, line: u32) {
        self.tokens.push(Token {
            token_type,
            literal: literal.to_string(),
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
                '(' => self.add_token(TokenType::LeftParen, "null", "(", line),
                ')' => self.add_token(TokenType::RightParen, "null", ")", line),
                '{' => self.add_token(TokenType::LeftBrace, "null", "{", line),
                '}' => self.add_token(TokenType::RightBrace, "null", "}", line),
                '*' => self.add_token(TokenType::Star, "null", "*", line),
                ';' => self.add_token(TokenType::Semicolon, "null", ";", line),
                '+' => self.add_token(TokenType::Plus, "null", "+", line),
                '-' => self.add_token(TokenType::Minus, "null", "-", line),
                '.' => self.add_token(TokenType::Dot, "null", ".", line),
                ',' => self.add_token(TokenType::Comma, "null", ",", line),
                '=' => {
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        self.add_token(TokenType::EqualEqual, "null", "==", line)
                    } else {
                        self.add_token(TokenType::Equal, "null", "=", line)
                    }
                }
                '!' => {
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        self.add_token(TokenType::BangEqual, "null", "!=", line)
                    } else {
                        self.add_token(TokenType::Bang, "null", "!", line)
                    }
                }
                '<' => {
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        self.add_token(TokenType::LessEqual, "null", "<=", line)
                    } else {
                        self.add_token(TokenType::LessThan, "null", "<", line)
                    }
                }
                '>' => {
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        self.add_token(TokenType::GreaterEqual, "null", ">=", line)
                    } else {
                        self.add_token(TokenType::GreaterThan, "null", ">", line)
                    }
                }
                '"' => {
                    let mut literal = String::from("\"");
                    let mut unterminated = false;
                    loop {
                        match chars.peek() {
                            Some(&'"') => {
                                chars.next();
                                break;
                            }

                            Some(&'\n') | None => {
                                has_error = true;
                                unterminated = true;
                                writeln!(
                                    io::stderr(),
                                    "[line {}] Error: Unterminated string.",
                                    line
                                )
                                .unwrap();
                                break;
                            }
                            Some(&ch) => {
                                literal.push(ch);
                                chars.next();
                            }
                        }
                    }
                    literal.push('"');
                    if !unterminated {
                        self.add_token(
                            TokenType::String,
                            &literal[1..literal.len() - 1],
                            &literal,
                            line,
                        );
                    }
                }
                '/' => {
                    if chars.peek() == Some(&'/') {
                        while chars.peek() != Some(&'\n') && chars.peek() != None {
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
