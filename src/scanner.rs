use crate::core::errors::ScanError;
use crate::core::literal::Literal;
use crate::core::token::{Token, TokenType};
use std::iter::Peekable;
use std::str::Chars;
pub struct Scanner<'a> {
    chars: Peekable<Chars<'a>>,
    line: u32,
    eof: bool,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().peekable(),
            line: 1,
            eof: false,
        }
    }

    fn tokenize_string(&mut self) -> Result<(TokenType, Literal, String), ScanError> {
        let mut lexeme = String::from('"');
        loop {
            match &self.peek() {
                Some('"') => {
                    lexeme.push('"');
                    self.advance();
                    break;
                }
                Some('\n') => {
                    lexeme.push('\n');
                    self.line += 1;
                    self.advance();
                }
                None => {
                    return Err(ScanError::UnterminatedString(self.line));
                }
                Some(&ch) => {
                    lexeme.push(ch);
                    self.advance();
                }
            }
        }

        Ok((
            TokenType::String,
            Literal::String(std::borrow::Cow::Owned(
                lexeme[1..lexeme.len() - 1].to_string(),
            )),
            lexeme,
        ))
    }

    fn tokenize_number(&mut self, init: char) -> Result<(TokenType, Literal, String), ScanError> {
        let mut lexeme = String::from(init);
        let mut has_decimal = false;

        while let Some(&d) = &self.peek() {
            if d == '.' {
                if has_decimal {
                    break;
                }

                if let Some(&next_char) = self.chars.peek() {
                    if next_char.is_ascii_digit() {
                        has_decimal = true;
                        lexeme.push('.');
                        self.advance();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else if d.is_ascii_digit() {
                lexeme.push(d);
                self.advance();
            } else {
                break;
            }
        }

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

        while let Some(&ch) = &self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                lexeme.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        Ok((
            match lexeme.as_str() {
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
            },
            Literal::Nil,
            lexeme,
        ))
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = &self.peek() {
            match c {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                _ => break,
            }
        }
    }

    fn skip_comment(&mut self, init: char) {
        if init == '/' && self.chars.peek() == Some(&'/') {
            self.advance();
            while self.peek() != Some(&'\n') && self.peek().is_some() {
                self.advance();
            }
        }
    }

    fn advance(&mut self) -> Option<char> {
        self.chars.next()
    }

    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }
}

impl Iterator for Scanner<'_> {
    type Item = Result<Token, ScanError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        let &c = match self.peek() {
            Some(c) => c,
            None => {
                if self.eof {
                    return None;
                } else {
                    self.eof = true;
                    return Some(Ok(Token {
                        token: TokenType::Eof,
                        lexeme: "".to_string(),
                        literal: Literal::Nil,
                        line: self.line,
                    }));
                }
            }
        };

        self.advance();
        self.skip_comment(c);

        let result = match c {
            '(' => Ok((TokenType::LeftParen, Literal::Nil, "(".to_string())),
            ')' => Ok((TokenType::RightParen, Literal::Nil, ")".to_string())),
            '{' => Ok((TokenType::LeftBrace, Literal::Nil, "{".to_string())),
            '}' => Ok((TokenType::RightBrace, Literal::Nil, "}".to_string())),
            '*' => Ok((TokenType::Star, Literal::Nil, "*".to_string())),
            ';' => Ok((TokenType::Semicolon, Literal::Nil, ";".to_string())),
            '+' => Ok((TokenType::Plus, Literal::Nil, "+".to_string())),
            '-' => Ok((TokenType::Minus, Literal::Nil, "-".to_string())),
            '.' => Ok((TokenType::Dot, Literal::Nil, ".".to_string())),
            ',' => Ok((TokenType::Comma, Literal::Nil, ",".to_string())),
            '/' => Ok((TokenType::Slash, Literal::Nil, "/".to_string())),
            '=' => {
                if self.peek() == Some(&'=') {
                    self.advance();
                    Ok((TokenType::EqualEqual, Literal::Nil, "==".to_string()))
                } else {
                    Ok((TokenType::Equal, Literal::Nil, "=".to_string()))
                }
            }
            '!' => {
                if self.peek() == Some(&'=') {
                    self.advance();
                    Ok((TokenType::BangEqual, Literal::Nil, "!=".to_string()))
                } else {
                    Ok((TokenType::Bang, Literal::Nil, "!".to_string()))
                }
            }
            '<' => {
                if self.peek() == Some(&'=') {
                    self.advance();
                    Ok((TokenType::LessEqual, Literal::Nil, "<=".to_string()))
                } else {
                    Ok((TokenType::LessThan, Literal::Nil, "<".to_string()))
                }
            }
            '>' => {
                if self.peek() == Some(&'=') {
                    self.advance();
                    Ok((TokenType::GreaterEqual, Literal::Nil, ">=".to_string()))
                } else {
                    Ok((TokenType::GreaterThan, Literal::Nil, ">".to_string()))
                }
            }
            '"' => self.tokenize_string(),
            d if d.is_ascii_digit() => self.tokenize_number(d),
            ch if ch.is_alphabetic() || ch == '_' => self.tokenize_identifier(ch),
            c => Err(ScanError::UnexpectedCharacter(self.line.to_owned(), c)),
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
