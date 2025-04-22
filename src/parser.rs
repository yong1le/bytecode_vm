use core::fmt;
use std::{iter::Peekable};

use crate::{
    scanner::{ScanError, Scanner},
    token::{Literal, Token, TokenType},
};

pub enum Expr {
    Literal(Literal),                      // NUMBER, STRING, true, false, nil
    Unary(Token, Box<Expr>),             // !, -
    Binary(Token, Box<Expr>, Box<Expr>), // +, -, *, /
    Grouping(Box<Expr>),                 // (, )
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Literal(l) => write!(f, "{}", l),
            Expr::Unary(token, expr) => write!(f, "{}{}", expr, token.lexeme),
            Expr::Binary(token, e1, e2) => write!(f, "{}{}{}", e1, token.lexeme, e2),
            Expr::Grouping(e) => write!(f, "({})", e),
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

    /** Advances to the next element in the iterator   */
    fn match_and_advance(&mut self, tokens: &[TokenType]) -> Option<Result<Token, ScanError>> {
        self.tokens.next_if(|_| match &self.current {
            Some(Ok(t)) => tokens.contains(&t.token),
            Some(Err(_)) => false,
            None => false,
        })
    }

    pub fn expression(&self) -> Expr {
        self.equality()
    }
    fn equality(&self) -> Expr {
        let mut left = self.comparison();

        return left;
    }
    fn comparison(&self) -> Expr {
        let mut left = self.term();
        return left;
    }
    fn term(&self) -> Expr {
        let mut left = self.factor();

        return left;
    }
    fn factor(&self) -> Expr {
        let mut left = self.unary();
        return left;
    }
    fn unary(&self) -> Expr {
        return self.primary();
    }
    fn primary(&self) -> Expr {
        match &self.current {
            Some(Ok(token)) => match &token.token {
                TokenType::True => Expr::Literal(Literal::Boolean(true)),
                TokenType::False => Expr::Literal(Literal::Boolean(false)),
                TokenType::Nil => Expr::Literal(Literal::Nil),
                TokenType::String => Expr::Literal(token.literal.to_owned()),
                _ => panic!("Not implemented")
            }
            Some(Err(_)) => panic!("Not implemented"),
            None => panic!("Not implemented")
        }
    }
}