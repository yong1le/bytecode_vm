use crate::token::{Literal, Token, TokenType};
use core::fmt;
use std::iter::Peekable;
use std::str::Chars;
pub struct Scanner<'a> {
    current: Option<char>,
    chars: Peekable<Chars<'a>>,
    line: u32,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut chars = source.chars().peekable();
        Self {
            current: chars.next(),
            chars,
            line: 1,
        }
    }

    fn tokenize_string(&mut self) -> Result<(TokenType, Literal, String), ScanError> {
        debug_assert_ne!(
            self.current,
            Some('"'),
            "tokenize_string called with quote current character {:?}",
            self.current
        );

        let mut lexeme = String::from('"');
        loop {
            match self.current {
                Some('"') => {
                    lexeme.push('"');
                    self.advance();
                    break;
                }

                Some('\n') | None => {
                    return Err(ScanError::UnterminatedString(self.line));
                }
                Some(ch) => {
                    lexeme.push(ch);
                    self.advance();
                }
            }
        }

        Ok((
            TokenType::String,
            Literal::String(lexeme[1..lexeme.len() - 1].to_string()),
            lexeme,
        ))
    }

    fn tokenize_number(&mut self, init: char) -> Result<(TokenType, Literal, String), ScanError> {
        let mut lexeme = String::from(init);
        let mut has_decimal = false;

        while let Some(d) = self.current {
            if d == '.' {
                if has_decimal {
                    break;
                }

                if let Some(&next_char) = self.chars.peek() {
                    if next_char.is_digit(10) {
                        has_decimal = true;
                        lexeme.push('.');
                        self.advance();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else if d.is_digit(10) {
                lexeme.push(d);
                self.advance();
            } else {
                break;
            }
        }

        debug_assert_ne!(self.current, Some('.'), "tokenize_number end with decimal");

        Ok((
            TokenType::Number,
            Literal::Number(lexeme.parse().unwrap()),
            lexeme,
        ))
    }

    fn tokenize_identifier(
        &mut self,
        init: char,
    ) -> Result<(TokenType, Literal, String), ScanError> {
        let mut lexeme = String::from(init);

        while let Some(ch) = self.current {
            if ch.is_alphanumeric() || ch == '_' {
                lexeme.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let token = match lexeme.as_str() {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fun" => TokenType::Fun,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Identifier,
        };

        if token == TokenType::Nil {
            Ok((token, Literal::Nil, lexeme))
        } else {
            Ok((token, Literal::None, lexeme))
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current {
            match c {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '/' if self.chars.peek() == Some(&'/') => {
                    self.advance();
                    self.advance();
                    while self.current != Some('\n') && self.current != None {
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    fn advance(&mut self) {
        self.current = self.chars.next();
    }
}

impl Iterator for Scanner<'_> {
    type Item = Result<Token, ScanError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        let &c = match &self.current {
            Some(c) => c,
            None => return None,
        };

        self.advance();

        let result = match c {
            '(' => Ok((TokenType::LeftParen, Literal::None, "(".to_string())),
            ')' => Ok((TokenType::RightParen, Literal::None, ")".to_string())),
            '{' => Ok((TokenType::LeftBrace, Literal::None, "{".to_string())),
            '}' => Ok((TokenType::RightBrace, Literal::None, "}".to_string())),
            '*' => Ok((TokenType::Star, Literal::None, "*".to_string())),
            ';' => Ok((TokenType::Semicolon, Literal::None, ";".to_string())),
            '+' => Ok((TokenType::Plus, Literal::None, "+".to_string())),
            '-' => Ok((TokenType::Minus, Literal::None, "-".to_string())),
            '.' => Ok((TokenType::Dot, Literal::None, ".".to_string())),
            ',' => Ok((TokenType::Comma, Literal::None, ",".to_string())),
            '/' => Ok((TokenType::Slash, Literal::None, "/".to_string())),
            '=' => {
                if self.current == Some('=') {
                    self.advance();
                    Ok((TokenType::EqualEqual, Literal::None, "==".to_string()))
                } else {
                    Ok((TokenType::Equal, Literal::None, "=".to_string()))
                }
            }
            '!' => {
                if self.current == Some('=') {
                    self.advance();
                    Ok((TokenType::BangEqual, Literal::None, "!=".to_string()))
                } else {
                    Ok((TokenType::Bang, Literal::None, "!".to_string()))
                }
            }
            '<' => {
                if self.current == Some('=') {
                    self.advance();
                    Ok((TokenType::LessEqual, Literal::None, "<=".to_string()))
                } else {
                    Ok((TokenType::LessThan, Literal::None, "<".to_string()))
                }
            }
            '>' => {
                if self.current == Some('=') {
                    self.advance();
                    Ok((TokenType::GreaterEqual, Literal::None, ">=".to_string()))
                } else {
                    Ok((TokenType::GreaterThan, Literal::None, ">".to_string()))
                }
            }
            '"' => self.tokenize_string(),
            d if d.is_digit(10) => self.tokenize_number(d),
            ch if ch.is_alphabetic() || ch == '_' => self.tokenize_identifier(ch),
            _ => Err(ScanError::UnexpectedCharacter(self.line, c)),
        };

        match result {
            Ok((token, literal, lexeme)) => Some(Ok(Token {
                token,
                literal,
                lexeme,
                line: self.line,
            })),
            Err(e) => Some(Err(e)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ScanError {
    UnterminatedString(u32),
    UnexpectedCharacter(u32, char),
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnterminatedString(line) => {
                write!(f, "[line {}] Error: Unterminated string.", line)
            }
            Self::UnexpectedCharacter(line, ch) => {
                write!(f, "[line {}] Error: Unexpected character: {}.", line, ch)
            }
        }
    }
}
