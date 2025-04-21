use std::fmt;
use std::io::{self, Write};
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LeftParen(u32),
    RightParen(u32),
    LeftBrace(u32),
    RightBrace(u32),
    Star(u32),
    Slash(u32),
    Semicolon(u32),
    Plus(u32),
    Minus(u32),
    Dot(u32),
    Comma(u32),
    Equal(u32),
    EqualEqual(u32),
    BangEqual(u32),
    Bang(u32),
    LessThan(u32),
    GreaterThan(u32),
    LessEqual(u32),
    GreaterEqual(u32),
    String(String, u32),
    Number(String, f64, u32),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::LeftParen(_) => write!(f, "LEFT_PAREN ( null"),
            Token::RightParen(_) => write!(f, "RIGHT_PAREN ) null"),
            Token::LeftBrace(_) => write!(f, "LEFT_BRACE {{ null"),
            Token::RightBrace(_) => write!(f, "RIGHT_BRACE }} null"),
            Token::Star(_) => write!(f, "STAR * null"),
            Token::Slash(_) => write!(f, "SLASH / null"),
            Token::Semicolon(_) => write!(f, "SEMICOLON ; null"),
            Token::Plus(_) => write!(f, "PLUS + null"),
            Token::Minus(_) => write!(f, "MINUS - null"),
            Token::Dot(_) => write!(f, "DOT . null"),
            Token::Comma(_) => write!(f, "COMMA , null"),
            Token::Equal(_) => write!(f, "EQUAL = null"),
            Token::EqualEqual(_) => write!(f, "EQUAL_EQUAL == null"),
            Token::BangEqual(_) => write!(f, "BANG_EQUAL != null"),
            Token::Bang(_) => write!(f, "BANG ! null"),
            Token::LessThan(_) => write!(f, "LESS < null"),
            Token::GreaterThan(_) => write!(f, "GREATER > null"),
            Token::LessEqual(_) => write!(f, "LESS_EQUAL <= null"),
            Token::GreaterEqual(_) => write!(f, "GREATER_EQUAL >= null"),
            Token::String(literal, _) => write!(f, "STRING \"{literal}\" {literal}"),
            Token::Number(lexeme, literal, _) => {
                let formatted_literal = if literal.fract() == 0.0 {
                    format!("{:.1}", literal)
                } else {
                    format!("{}", literal)
                };
                write!(f, "NUMBER {lexeme} {formatted_literal}")
            }
        }
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

    pub fn print(&self) {
        self.tokens.iter().for_each(|t| println!("{t}"));
        println!("EOF  null")
    }

    fn skip_line(&self, chars: &mut Peekable<Chars<'_>>) {}

    fn tokenize_string(&mut self, chars: &mut Peekable<Chars<'_>>, line: u32) -> Result<(), ()> {
        let mut literal = String::new();
        loop {
            match chars.peek() {
                Some(&'"') => {
                    chars.next();
                    break;
                }

                Some(&'\n') | None => {
                    return Err(());
                }
                Some(&ch) => {
                    literal.push(ch);
                    chars.next();
                }
            }
        }
        self.tokens.push(Token::String(literal, line));
        Ok(())
    }

    fn tokenize_number(
        &mut self,
        init: char,
        chars: &mut Peekable<Chars<'_>>,
        line: u32,
    ) -> Result<(), ()> {
        let mut literal = String::from(init);
        let mut has_decimal = false;
        loop {
            match chars.peek() {
                Some(&'.') => {
                    if has_decimal {
                        break;
                    }
                    chars.next();
                    if let Some(&ch) = chars.peek() {
                        if ch.is_digit(10) {
                            literal.push('.');
                            has_decimal = true;
                        } else {
                            return Err(());
                        }
                    } else {
                        return Err(());
                    }
                }
                Some(&'\n') | None => {
                    chars.next();
                    break;
                }
                Some(&d) if d.is_digit(10) => {
                    literal.push(d);
                    chars.next();
                }
                Some(&ch) => {
                    return Err(());
                }
            }
        }
        if literal.ends_with('.') {
            self.tokens.push(Token::Number(
                literal[..literal.len() - 1].to_string(),
                literal.parse().unwrap(),
                line,
            ));
            self.tokens.push(Token::Dot(line))
        } else {
            self.tokens.push(Token::Number(
                literal.to_string(),
                literal.parse().unwrap(),
                line,
            ));
        }

        Ok(())
    }

    pub fn tokenize(&mut self) -> Result<(), ()> {
        let mut has_error = false;
        let mut chars = self.source.chars().peekable();
        let mut line = 1u32;

        while let Some(c) = chars.next() {
            match c {
                '(' => self.tokens.push(Token::LeftParen(line)),
                ')' => self.tokens.push(Token::RightParen(line)),
                '{' => self.tokens.push(Token::LeftBrace(line)),
                '}' => self.tokens.push(Token::RightBrace(line)),
                '*' => self.tokens.push(Token::Star(line)),
                ';' => self.tokens.push(Token::Semicolon(line)),
                '+' => self.tokens.push(Token::Plus(line)),
                '-' => self.tokens.push(Token::Minus(line)),
                '.' => self.tokens.push(Token::Dot(line)),
                ',' => self.tokens.push(Token::Comma(line)),
                '=' => {
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        self.tokens.push(Token::EqualEqual(line))
                    } else {
                        self.tokens.push(Token::Equal(line))
                    }
                }
                '!' => {
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        self.tokens.push(Token::BangEqual(line))
                    } else {
                        self.tokens.push(Token::Bang(line))
                    }
                }
                '<' => {
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        self.tokens.push(Token::LessEqual(line))
                    } else {
                        self.tokens.push(Token::LessThan(line))
                    }
                }
                '>' => {
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        self.tokens.push(Token::GreaterEqual(line))
                    } else {
                        self.tokens.push(Token::GreaterThan(line))
                    }
                }
                '"' => self.tokenize_string(&mut chars, line).unwrap_or_else(|_| {
                    has_error = true;
                    writeln!(io::stderr(), "[line {}] Error: Unterminated string.", line).unwrap();
                }),
                d if d.is_digit(10) => {
                    self.tokenize_number(d, &mut chars, line)
                        .unwrap_or_else(|_| {
                            has_error = true;
                            writeln!(
                                io::stderr(),
                                "[line {}] Error: Invalided integer literal.",
                                line
                            )
                            .unwrap();
                        })
                }
                '/' => {
                    if chars.peek() == Some(&'/') {
                        while chars.peek() != Some(&'\n') && chars.peek() != None {
                            chars.next();
                        }
                    } else {
                        self.tokens.push(Token::Slash(line))
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
