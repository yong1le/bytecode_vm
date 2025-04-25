use crate::core::errors::ScanError;
use crate::core::literal::Literal;
use crate::core::token::{Token, TokenType};
use std::iter::Peekable;
use std::str::Chars;

/// An iterator over the tokens in the source code.
pub struct Scanner<'a> {
    /// An iterator over the characters in the source code.
    chars: Peekable<Chars<'a>>,
    /// The current line number processed to in the source code.
    line: u32,
    /// Whether the end of the file has been reached.
    eof: bool,
    /// Temporary store for a character that was skipped over.
    unget: Option<char>,
}

impl<'a> Scanner<'a> {
    /// Creates a new scanner for the given source code.
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().peekable(),
            line: 1,
            eof: false,
            unget: None,
        }
    }

    /// Tokenizes a string from the source code.
    ///
    /// Returns a `ScanError::UnterminatedString` if the string is not terminated.
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

    /// Tokenizes a number from the source code.
    ///
    /// Numbers cannot be preceded by decimals nor can they be end with a decimal.
    fn tokenize_number(&mut self, init: char) -> Result<(TokenType, Literal, String), ScanError> {
        let mut lexeme = String::from(init);
        let mut has_decimal = false;

        while let Some(&d) = self.peek() {
            if d == '.' {
                if has_decimal {
                    break;
                }

                self.advance(); // skips the decimal point
                if let Some(&next_char) = self.peek() {
                    if next_char.is_ascii_digit() {
                        has_decimal = true;
                        lexeme.push('.');
                    } else {
                        self.unget = Some('.');
                        break;
                    }
                } else {
                    self.unget = Some('.');
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

    /// Tokenizes an identifier from the source code.
    ///
    /// Identifiers can contain letters, digits, and underscores.
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

    /// Skips over all whitespace and comments in the source code.
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
                '/' => {
                    self.advance(); // skips over first '/'
                    match self.peek() {
                        // if the second character is also a '/'
                        Some(&'/') => {
                            self.advance(); // skips over the second '/'
                            while self.peek() != Some(&'\n') && self.peek().is_some() {
                                self.advance();
                            }
                        }
                        _ => {
                            self.unget = Some('/');
                            break;
                        }
                    }
                }
                _ => break,
            }
        }
    }

    /// Advance the internal character iterator by one character. If there is some value
    /// in `self.unget`, return that value instead.
    fn advance(&mut self) -> Option<char> {
        if self.unget.is_some() {
            let unget = self.unget;
            self.unget = None;
            unget
        } else {
            self.chars.next()
        }
    }

    /// Peeks at the next character in the source code without consuming it.
    fn peek(&mut self) -> Option<&char> {
        if self.unget.is_some() {
            self.unget.as_ref()
        } else {
            self.chars.peek()
        }
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
