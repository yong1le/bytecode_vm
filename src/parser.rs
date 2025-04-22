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
            Expr::Binary(token, e1, e2) => write!(f, "{}{}{}", e1, token.lexeme, e2),
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
     * in tokens. Otherwise, panic.
     */
    fn match_and_advance(&mut self, tokens: &[TokenType]) {
        match &self.current {
            Some(Ok(t)) => {
                if !tokens.contains(&t.token) {
                    panic!(
                        "[line {}] Error: Expected one of the following: {:?}, received {}. ",
                        t.line, tokens, t.lexeme
                    );
                }
            }
            Some(Err(e)) => panic!("{}", e),
            None => panic!(
                "Error: Expected one of the following: {:?}, received EOF. ",
                tokens
            ),
        }

        self.advance();
    }

    pub fn expression(&mut self) -> Expr {
        self.equality()
    }
    fn equality(&mut self) -> Expr {
        let mut left = self.comparison();

        return left;
    }
    fn comparison(&mut self) -> Expr {
        let mut left = self.term();
        return left;
    }
    fn term(&mut self) -> Expr {
        let mut left = self.factor();

        return left;
    }
    fn factor(&mut self) -> Expr {
        let mut left = self.unary();
        return left;
    }
    fn unary(&mut self) -> Expr {
        let t = match &self.current {
            Some(Ok(t)) => t.to_owned(),
            Some(Err(e)) => panic!("{}", e),
            None => panic!("END"),
        };

        match t.token {
            TokenType::Bang | TokenType::Minus => {
                self.advance();
                let expr = self.unary();
                Expr::Unary(t, Box::new(expr))
            }
            _ => self.primary(),
        }
    }
    fn primary(&mut self) -> Expr {
        let t = match &self.current {
            Some(Ok(t)) => t.to_owned(),
            Some(Err(e)) => panic!("{}", e),
            None => panic!("END"),
        };

        self.advance();

        match t.token {
            TokenType::True => Expr::Literal(Literal::Boolean(true)),
            TokenType::False => Expr::Literal(Literal::Boolean(false)),
            TokenType::Nil => Expr::Nil,
            TokenType::String => Expr::Literal(t.literal.to_owned()),
            TokenType::Number => Expr::Literal(t.literal.to_owned()),
            TokenType::LeftParen => {
                let expr = self.expression();
                self.match_and_advance(&vec![TokenType::RightParen]);
                Expr::Grouping(Box::new(expr))
            }
            _ => panic!(
                "[line {}] Did not expect {} {} here.",
                t.line, t.token, t.lexeme
            ),
        }
    }
}
