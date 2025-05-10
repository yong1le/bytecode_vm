use crate::core::errors::{InterpretError, ScanError};
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
    fn tokenize_string(&mut self) -> Result<(TokenType, String), InterpretError> {
        let mut lexeme = String::from('"');
        loop {
            match self.peek() {
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
                    println!("HERE");
                    return Err(InterpretError::Scan(ScanError::UnterminatedString(
                        self.line,
                    )));
                }
                Some(&ch) => {
                    lexeme.push(ch);
                    self.advance();
                }
            }
        }

        Ok((TokenType::String, lexeme))
    }

    /// Tokenizes a number from the source code.
    ///
    /// Numbers cannot be preceded by decimals nor can they be end with a decimal.
    fn tokenize_number(&mut self, init: char) -> Result<(TokenType, String), InterpretError> {
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

        Ok((TokenType::Number, lexeme))
    }

    /// Tokenizes an identifier from the source code.
    ///
    /// Identifiers can contain letters, digits, and underscores.
    fn tokenize_identifier(&mut self, init: char) -> Result<(TokenType, String), InterpretError> {
        let mut lexeme = String::from(init);

        while let Some(&ch) = self.peek() {
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
            lexeme,
        ))
    }

    /// Skips over all whitespace and comments in the source code.
    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.peek() {
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

    fn add_token(&mut self, token: TokenType, lexeme: String, line: u32) -> Token {
        Token {
            token,
            lexeme,
            line,
        }
    }
}

impl Iterator for Scanner<'_> {
    type Item = Result<Token, InterpretError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        let &c = match self.peek() {
            Some(c) => c,
            None => {
                if self.eof {
                    return None;
                } else {
                    self.eof = true;
                    return Some(Ok(self.add_token(
                        TokenType::Eof,
                        "".to_string(),
                        self.line,
                    )));
                }
            }
        };

        self.advance();

        let result = match c {
            '(' => Ok((TokenType::LeftParen, "(".to_string())),
            ')' => Ok((TokenType::RightParen, ")".to_string())),
            '{' => Ok((TokenType::LeftBrace, "{".to_string())),
            '}' => Ok((TokenType::RightBrace, "}".to_string())),
            '*' => Ok((TokenType::Star, "*".to_string())),
            ';' => Ok((TokenType::Semicolon, ";".to_string())),
            '+' => Ok((TokenType::Plus, "+".to_string())),
            '-' => Ok((TokenType::Minus, "-".to_string())),
            '.' => Ok((TokenType::Dot, ".".to_string())),
            ',' => Ok((TokenType::Comma, ",".to_string())),
            '/' => Ok((TokenType::Slash, "/".to_string())),
            '=' => {
                if self.peek() == Some(&'=') {
                    self.advance();
                    Ok((TokenType::EqualEqual, "==".to_string()))
                } else {
                    Ok((TokenType::Equal, "=".to_string()))
                }
            }
            '!' => {
                if self.peek() == Some(&'=') {
                    self.advance();
                    Ok((TokenType::BangEqual, "!=".to_string()))
                } else {
                    Ok((TokenType::Bang, "!".to_string()))
                }
            }
            '<' => {
                if self.peek() == Some(&'=') {
                    self.advance();
                    Ok((TokenType::LessEqual, "<=".to_string()))
                } else {
                    Ok((TokenType::LessThan, "<".to_string()))
                }
            }
            '>' => {
                if self.peek() == Some(&'=') {
                    self.advance();
                    Ok((TokenType::GreaterEqual, ">=".to_string()))
                } else {
                    Ok((TokenType::GreaterThan, ">".to_string()))
                }
            }
            '"' => self.tokenize_string(),
            d if d.is_ascii_digit() => self.tokenize_number(d),
            ch if ch.is_alphabetic() || ch == '_' => self.tokenize_identifier(ch),
            c => Err(InterpretError::Scan(ScanError::UnexpectedCharacter(
                self.line.to_owned(),
                c,
            ))),
        };

        match result {
            Ok((token, lexeme)) => Some(Ok(self.add_token(token, lexeme, self.line))),
            Err(e) => Some(Err(e)),
        }
    }
}
