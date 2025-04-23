use core::fmt;

use crate::token::{Literal, Token};

pub enum Expr {
    Literal(Literal),                    // NUMBER, STRING, true, false, nil
    Unary(Token, Box<Expr>),             // !, -
    Binary(Token, Box<Expr>, Box<Expr>), // +, -, *, /, <, <=, >, >=
    Grouping(Box<Expr>),                 // (, )
    Variable(String),
}

pub trait ExprVisitor<T> {
    fn visit_literal(&mut self, literal: &Literal) -> T;
    fn visit_unary(&mut self, operator: &Token, expr: &Expr) -> T;
    fn visit_binary(&mut self, operator: &Token, left: &Expr, right: &Expr) -> T;
    fn visit_grouping(&mut self, expr: &Expr) -> T;
    fn visit_variable(&mut self, id: &String) -> T;
}

impl Expr {
    pub fn accept<T>(&self, visitor: &mut impl ExprVisitor<T>) -> T {
        match self {
            Expr::Literal(literal) => visitor.visit_literal(literal),
            Expr::Unary(op, expr) => visitor.visit_unary(op, expr),
            Expr::Binary(op, left, right) => visitor.visit_binary(op, left, right),
            Expr::Grouping(expr) => visitor.visit_grouping(expr),
            Expr::Variable(id) => visitor.visit_variable(id),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Literal(l) => write!(f, "{}", l),
            Expr::Unary(token, expr) => write!(f, "({} {})", token.lexeme, expr),
            Expr::Binary(token, e1, e2) => write!(f, "({} {} {})", token.lexeme, e1, e2),
            Expr::Grouping(e) => write!(f, "(group {})", e),
            Expr::Variable(id) => write!(f, "({})", id),
        }
    }
}
