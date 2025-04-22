use core::fmt;
use std::iter::Peekable;

use crate::{
    scanner::{ScanError, Scanner},
    token::{self, Literal, Token, TokenType},
};

pub enum Expr {
    Nil,                                 // nil
    Literal(Literal),                    // NUMBER, STRING, true, false,
    Unary(Token, Box<Expr>),             // !, -
    Binary(Token, Box<Expr>, Box<Expr>), // +, -, *, /
    Grouping(Box<Expr>),                 // (, )
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Nil => write!(f, "nil"),
            Expr::Literal(l) => write!(f, "{}", l),
            Expr::Unary(token, expr) => write!(f, "({} {})", token.lexeme, expr),
            Expr::Binary(token, e1, e2) => write!(f, "({} {} {})", token.lexeme, e1, e2),
            Expr::Grouping(e) => write!(f, "(group {})", e),
        }
    }
}

// expression     → equality ;
// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term           → factor ( ( "-" | "+" ) factor )* ;
// factor         → unary ( ( "/" | "*" ) unary )* ;
// unary          → ( "!" | "-" ) unary
//                | primary ;
// primary        → NUMBER | STRING | "true" | "false" | "nil"
//                | "(" expression ")" ;

pub struct Parser<'a> {
    current: Option<Result<Token, ScanError>>,
    tokens: Peekable<Scanner<'a>>,
}

impl Parser<'_> {
    pub fn new(tokens: Scanner<'_>) -> Parser {
        let mut tokens = tokens.into_iter().peekable();
        return Parser {
            current: tokens.next(),
            tokens,
        };
    }

    fn advance(&mut self) {
        self.current = self.tokens.next();
    }

    /** Advances the iterator if the current element matches on of the TokenTypes
     * in tokens. Otherwise, returns.
     */
    fn match_and_advance(&mut self, tokens: &[TokenType]) -> Result<(), String> {
        match &self.current {
            Some(Ok(t)) => {
                if !tokens.contains(&t.token) {
                    return Err(t.lexeme.to_string());
                }
            }
            Some(Err(e)) => panic!("{}", e),
            None => return Err("EOF".to_string()),
        }

        self.advance();
        Ok(())
    }

    pub fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        loop {
            let t = match &self.current {
                Some(Ok(t)) => t.to_owned(),
                Some(Err(e)) => return Err(ParseError::ScanError(e.to_owned())),
                None => return Ok(expr),
            };

            match t.token {
                TokenType::EqualEqual | TokenType::BangEqual => {
                    self.advance();
                    let right = self.comparison()?;
                    expr = Expr::Binary(t, Box::new(expr), Box::new(right))
                }
                _ => break,
            }
        }

        return Ok(expr);
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;

        loop {
            let t = match &self.current {
                Some(Ok(t)) => t.to_owned(),
                Some(Err(e)) => return Err(ParseError::ScanError(e.to_owned())),
                None => return Ok(expr),
            };

            match t.token {
                TokenType::LessEqual
                | TokenType::LessThan
                | TokenType::GreaterEqual
                | TokenType::GreaterThan => {
                    self.advance();
                    let right = self.term()?;
                    expr = Expr::Binary(t, Box::new(expr), Box::new(right))
                }
                _ => break,
            }
        }

        return Ok(expr);
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        loop {
            let t = match &self.current {
                Some(Ok(t)) => t.to_owned(),
                Some(Err(e)) => return Err(ParseError::ScanError(e.to_owned())),
                None => return Ok(expr),
            };

            match t.token {
                TokenType::Plus | TokenType::Minus => {
                    self.advance();
                    let right = self.factor()?;
                    expr = Expr::Binary(t, Box::new(expr), Box::new(right))
                }
                _ => break,
            }
        }

        return Ok(expr);
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        loop {
            let t = match &self.current {
                Some(Ok(t)) => t.to_owned(),
                Some(Err(e)) => return Err(ParseError::ScanError(e.to_owned())),
                None => return Ok(expr),
            };

            match t.token {
                TokenType::Star | TokenType::Slash => {
                    self.advance();
                    let right = self.unary()?;
                    expr = Expr::Binary(t, Box::new(expr), Box::new(right))
                }
                _ => break,
            }
        }

        return Ok(expr);
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        let t = match &self.current {
            Some(Ok(t)) => t.to_owned(),
            Some(Err(e)) => return Err(ParseError::ScanError(e.to_owned())),
            None => return Err(ParseError::UnexpectedEOF),
        };

        match t.token {
            TokenType::Bang | TokenType::Minus => {
                self.advance();
                let expr = self.unary()?;
                Ok(Expr::Unary(t, Box::new(expr)))
            }
            _ => self.primary(),
        }
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        let t = match &self.current {
            Some(Ok(t)) => t.to_owned(),
            Some(Err(e)) => return Err(ParseError::ScanError(e.to_owned())),
            None => return Err(ParseError::UnexpectedEOF),
        };

        self.advance();

        let expr = match t.token {
            TokenType::True => Expr::Literal(Literal::Boolean(true)),
            TokenType::False => Expr::Literal(Literal::Boolean(false)),
            TokenType::Nil => Expr::Nil,
            TokenType::String => Expr::Literal(t.literal.to_owned()),
            TokenType::Number => Expr::Literal(t.literal.to_owned()),
            TokenType::LeftParen => {
                let expr = self.expression()?;
                match self.match_and_advance(&vec![TokenType::RightParen]) {
                    Ok(()) => (),
                    Err(e) => return Err(ParseError::ExpectedParen(t.line, e)),
                };
                Expr::Grouping(Box::new(expr))
            }
            _ => return Err(ParseError::ExpectedExpression(t.line, t.lexeme)),
        };

        Ok(expr)
    }
}

#[derive(Debug, Clone)]
pub enum ParseError {
    ScanError(ScanError),
    ExpectedParen(u32, String),
    ExpectedExpression(u32, String),
    UnexpectedEOF,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ScanError(s) => {
                write!(f, "{s}")
            }
            Self::ExpectedParen(line, actual) => {
                write!(
                    f,
                    "[line {}] Error at {}: Expect ')'.",
                    line, actual
                )
            }
            Self::ExpectedExpression(line, actual) => {
                write!(f, "[line {}] Error at '{}': Expect expression.", line, actual)
            }

            Self::UnexpectedEOF => {
                write!(f, "Error: Unexpected End of File.")
            }
        }
    }
}
