use core::fmt;

use crate::token::Token;

use super::expr::Expr;

#[derive(Debug, Clone)]
pub enum Stmt {
    Print(Expr),
    Expr(Expr),
    DeclareVar(Token, Expr),
    Block(Vec<Stmt>),
}

pub trait StmtVisitor<T> {
    fn visit_print(&mut self, stmt: &Expr) -> T;
    fn visit_expr(&mut self, expr: &Expr) -> T;
    fn visit_declare_var(&mut self, id: &Token, expr: &Expr) -> T;
    fn visit_block(&mut self, statements: &Vec<Stmt>) -> T;
}

impl Stmt {
    pub fn accept<T>(&self, visiter: &mut impl StmtVisitor<T>) -> T {
        match self {
            Stmt::Print(expr) => visiter.visit_print(expr),
            Stmt::Expr(expr) => visiter.visit_expr(expr),
            Stmt::DeclareVar(id, expr) => visiter.visit_declare_var(id, expr),
            Stmt::Block(statements) => visiter.visit_block(statements),
        }
    }
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Expr(e) => write!(f, "{}", e),
            Stmt::Print(e) => write!(f, "(print {})", e),
            Stmt::DeclareVar(id, expr) => write!(f, "(var {} ({}))", id.lexeme, expr),
            Stmt::Block(stmts) => write!(f, "(block {:?})", stmts),
        }
    }
}
