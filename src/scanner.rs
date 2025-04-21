use std::fmt::{self};
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
    Identifier(String, u32),

    And(u32),
    Class(u32),
    Else(u32),
    False(u32),
    For(u32),
    Fun(u32),
    If(u32),
    Nil(u32),
    Or(u32),
    Print(u32),
    Return(u32),
    Super(u32),
    This(u32),
    True(u32),
    Var(u32),
    While(u32),
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
            Token::Identifier(identifier, _) => write!(f, "IDENTIFIER {identifier} null"),
            Token::And(_) => write!(f, "AND and null"),
            Token::Class(_) => write!(f, "CLASS class null"),
            Token::Else(_) => write!(f, "ELSE else null"),
            Token::False(_) => write!(f, "FALSE false null"),
            Token::For(_) => write!(f, "FOR for null"),
            Token::Fun(_) => write!(f, "FUN fun null"),
            Token::If(_) => write!(f, "IF if null"),
            Token::Nil(_) => write!(f, "NIL nil null"),
            Token::Or(_) => write!(f, "OR or null"),
            Token::Print(_) => write!(f, "PRINT print null"),
            Token::Return(_) => write!(f, "RETURN return null"),
            Token::Super(_) => write!(f, "SUPER super null"),
            Token::This(_) => write!(f, "THIS this null"),
            Token::True(_) => write!(f, "TRUE true null"),
            Token::Var(_) => write!(f, "VAR var null"),
            Token::While(_) => write!(f, "WHILE while null"),
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
        let mut lexeme = String::from(init);
        let mut has_decimal = false;
        loop {
            match chars.peek() {
                Some(&'.') => {
                    if has_decimal {
                        break;
                    }
                    lexeme.push('.');
                    chars.next();
                    if let Some(&ch) = chars.peek() {
                        if ch.is_digit(10) {
                            has_decimal = true;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                Some(&d) if d.is_digit(10) => {
                    lexeme.push(d);
                    chars.next();
                }
                Some(_) | None => {
                    break;
                }
            }
        }
        if lexeme.ends_with('.') {
            self.tokens.push(Token::Number(
                lexeme[..lexeme.len() - 1].to_string(),
                lexeme.parse().unwrap(),
                line,
            ));
            self.tokens.push(Token::Dot(line))
        } else {
            self.tokens.push(Token::Number(
                lexeme.to_string(),
                lexeme.parse().unwrap(),
                line,
            ));
        }

        Ok(())
    }

    fn tokenize_identifier(
        &mut self,
        init: char,
        chars: &mut Peekable<Chars<'_>>,
        line: u32,
    ) -> Result<(), ()> {
        let mut lexeme = String::from(init);

        loop {
            match chars.peek() {
                Some(&ch) if ch.is_alphanumeric() || ch == '_' => {
                    chars.next();
                    lexeme.push(ch);
                }
                Some(_) | None => {
                    break;
                }
            };
        }

        match lexeme.as_str() {
            "and" => self.tokens.push(Token::And(line)),
            "class" => self.tokens.push(Token::Class(line)),
            "else" => self.tokens.push(Token::Else(line)),
            "false" => self.tokens.push(Token::False(line)),
            "for" => self.tokens.push(Token::For(line)),
            "fun" => self.tokens.push(Token::Fun(line)),
            "if" => self.tokens.push(Token::If(line)),
            "nil" => self.tokens.push(Token::Nil(line)),
            "or" => self.tokens.push(Token::Or(line)),
            "print" => self.tokens.push(Token::Print(line)),
            "return" => self.tokens.push(Token::Return(line)),
            "super" => self.tokens.push(Token::Super(line)),
            "this" => self.tokens.push(Token::This(line)),
            "true" => self.tokens.push(Token::True(line)),
            "var" => self.tokens.push(Token::Var(line)),
            "while" => self.tokens.push(Token::While(line)),
            _ => {
                self.tokens.push(Token::Identifier(lexeme, line));
            }
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
                            writeln!(io::stderr(), "[line {}] Error: Invalided integer.", line)
                                .unwrap();
                        })
                }
                ch if ch.is_alphabetic() || ch == '_' => {
                    self.tokenize_identifier(ch, &mut chars, line)
                        .unwrap_or_else(|_| {
                            has_error = true;
                            writeln!(io::stderr(), "[line {}] Error: Invalided identifier.", line)
                                .unwrap();
                        });
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
