pub mod expr;
pub mod stmt;

use core::fmt;
use std::{iter::Peekable, vec};

use expr::Expr;
use stmt::Stmt;

use crate::{
    scanner::{ScanError, Scanner},
    token::{Literal, Token, TokenType},
};

// program        →  declaration* EOF;
// declaration    -> varDecl | statement;
// varDecl        -> "var" IDENTIFIER ( "=" )? ";";
// statement      → exprStmt | printStmt ;
// exprStmt       → expression ";" '
// printStmt      → "print" expression ";" ;

// expression     → equality ;
// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term           → factor ( ( "-" | "+" ) factor )* ;
// factor         → unary ( ( "/" | "*" ) unary )* ;
// unary          → ( "!" | "-" ) unary
//                | primary ;
// primary        → NUMBER | STRING | "true" | "false" | "nil"
//                | "(" expression ")" | IDENTIFIER;

pub struct Parser<'a> {
    current: Option<Result<Token, ScanError>>,
    tokens: Peekable<Scanner<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Scanner<'a>) -> Self {
        let mut tokens = tokens.into_iter().peekable();
        Self {
            current: tokens.next(),
            tokens,
        }
    }

    /// Parses the first expression from the list of tokens and advances the
    /// iterator.
    pub fn parse(&mut self) -> Option<Result<Expr, ParseError>> {
        if self.current.is_none() {
            return None;
        }

        let result = self.expression();
        match result {
            Err(ParseError::ScanError(_)) => self.advance(),
            _ => (),
        }

        // If the first expression was instead an expression statement
        if let Some(Ok(token)) = &self.current {
            if token.token == TokenType::Semicolon {
                self.advance();
            }
        }

        Some(result)
    }

    fn advance(&mut self) {
        self.current = self.tokens.next();
    }

    /** Advances the iterator if the current element matches on of the TokenTypes
     * in tokens. Otherwise, returns.
     */
    fn match_and_advance(&mut self, tokens: &[TokenType]) -> Result<(), ParseError> {
        match &self.current {
            Some(Ok(t)) => {
                if !tokens.contains(&t.token) {
                    return Err(ParseError::ExpectedChar(
                        t.line,
                        t.lexeme.to_string(),
                        format!("{:?}", tokens),
                    ));
                }
            }
            Some(Err(e)) => panic!("{}", e),
            None => return Err(ParseError::UnexpectedEOF),
        }

        self.advance();
        Ok(())
    }

    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        let t = match &self.current {
            Some(Ok(t)) => t.to_owned(),
            Some(Err(e)) => return Err(ParseError::ScanError(e.to_owned())),
            None => return Err(ParseError::UnexpectedEOF),
        };

        match t.token {
            TokenType::Var => {
                self.advance();
                self.variable()
            }
            _ => self.statement(),
        }
    }

    fn variable(&mut self) -> Result<Stmt, ParseError> {
        let t = match &self.current {
            Some(Ok(t)) => t.to_owned(),
            Some(Err(e)) => return Err(ParseError::ScanError(e.to_owned())),
            None => return Err(ParseError::UnexpectedEOF),
        };

        match t.token {
            TokenType::Identifier => {
                self.advance();
                self.match_and_advance(&vec![TokenType::Equal])?;
                let initializer = self.expression()?;
                self.match_and_advance(&vec![TokenType::Semicolon])?;
                Ok(Stmt::Variable(t.lexeme, initializer))
            }
            _ => Err(ParseError::ExpectedChar(t.line, t.lexeme.to_string(), "IDENTIFIER".to_string()))
        }
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        let t = match &self.current {
            Some(Ok(t)) => t.to_owned(),
            Some(Err(e)) => return Err(ParseError::ScanError(e.to_owned())),
            None => return Err(ParseError::UnexpectedEOF),
        };

        match t.token {
            TokenType::Print => {
                self.advance();
                self.print()
            }
            _ => self.expression_stmt(),
        }
    }

    fn print(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        self.match_and_advance(&vec![TokenType::Semicolon])?;

        Ok(Stmt::Print(expr))
    }

    /* Basically expression, but consume the semicolon */
    fn expression_stmt(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        self.match_and_advance(&vec![TokenType::Semicolon])?;
        Ok(Stmt::Expr(expr))
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
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
            TokenType::Identifier => Expr::Variable(t.lexeme),
            TokenType::True => Expr::Literal(Literal::Boolean(true)),
            TokenType::False => Expr::Literal(Literal::Boolean(false)),
            TokenType::Nil => Expr::Literal(Literal::Nil),
            TokenType::String => Expr::Literal(t.literal.to_owned()),
            TokenType::Number => Expr::Literal(t.literal.to_owned()),
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.match_and_advance(&vec![TokenType::RightParen])?;
                Expr::Grouping(Box::new(expr))
            }
            _ => return Err(ParseError::ExpectedExpression(t.line, t.lexeme)),
        };

        Ok(expr)
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Stmt, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_none() {
            return None;
        }

        let result = self.declaration();
        match result {
            Err(ParseError::ScanError(_)) => self.advance(),
            _ => (),
        }

        Some(result)
    }
}
#[derive(Debug, Clone)]
pub enum ParseError {
    ScanError(ScanError),
    ExpectedChar(u32, String, String),
    ExpectedExpression(u32, String),
    UnexpectedEOF,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ScanError(s) => {
                write!(f, "{s}")
            }
            Self::ExpectedChar(line, actual, expected) => {
                write!(
                    f,
                    "[line {}] Error at '{}': Expected {}.",
                    line, actual, expected
                )
            }
            Self::ExpectedExpression(line, actual) => {
                write!(
                    f,
                    "[line {}] Error at '{}': Expected expression.",
                    line, actual
                )
            }

            Self::UnexpectedEOF => {
                write!(f, "Error: Unexpected End of File.")
            }
        }
    }
}
