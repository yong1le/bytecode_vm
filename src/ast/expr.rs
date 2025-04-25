use core::fmt;

use crate::core::{literal::Literal, token::Token};

/// Enum to represent different types of expressions in the AST.
#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),                    // NUMBER, STRING, true, false, nil
    Unary(Token, Box<Expr>),             // !, -
    Binary(Token, Box<Expr>, Box<Expr>), // +, -, *, /, <, <=, >, >=
    Grouping(Box<Expr>),                 // (, )
    Variable(Token),
    Assign(Token, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
}

/// A struct that visits `Expr`
pub trait ExprVisitor<T> {
    fn visit_literal(&mut self, literal: &Literal) -> T;
    fn visit_unary(&mut self, operator: &Token, expr: &Expr) -> T;
    fn visit_binary(&mut self, operator: &Token, left: &Expr, right: &Expr) -> T;
    fn visit_grouping(&mut self, expr: &Expr) -> T;
    fn visit_variable(&mut self, id: &Token) -> T;
    fn visit_assignment(&mut self, id: &Token, assignment: &Expr) -> T;
    fn visit_and(&mut self, left: &Expr, right: &Expr) -> T;
    fn visit_or(&mut self, left: &Expr, right: &Expr) -> T;
}

impl Expr {
    pub fn accept<T>(&self, visitor: &mut impl ExprVisitor<T>) -> T {
        match self {
            Expr::Literal(literal) => visitor.visit_literal(literal),
            Expr::Unary(op, expr) => visitor.visit_unary(op, expr),
            Expr::Binary(op, left, right) => visitor.visit_binary(op, left, right),
            Expr::Grouping(expr) => visitor.visit_grouping(expr),
            Expr::Variable(id) => visitor.visit_variable(id),
            Expr::Assign(id, assignment) => visitor.visit_assignment(id, assignment),
            Expr::And(left, right) => visitor.visit_and(left, right),
            Expr::Or(left, right) => visitor.visit_or(left, right),
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
            Expr::Variable(id) => write!(f, "({})", id.lexeme),
            Expr::Assign(id, expr) => write!(f, "({} ({}))", id.lexeme, expr),
            Expr::And(e1, e2) => write!(f, "(and {} {})", e1, e2),
            Expr::Or(e1, e2) => write!(f, "(or {} {})", e1, e2),
        }
    }
}
