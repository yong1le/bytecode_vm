use core::fmt;

use crate::core::token::Token;

use super::expr::Expr;

/// Enum to represent different types of statements in the AST.
#[derive(Debug, Clone)]
pub enum Stmt {
    Print(Expr),
    Expr(Expr),
    DeclareVar(Token, Expr),
    Block(Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
}

/// A struct that visits `Stmt`
pub trait StmtVisitor<T> {
    fn visit_print(&mut self, stmt: &Expr) -> T;
    fn visit_expr(&mut self, expr: &Expr) -> T;
    fn visit_declare_var(&mut self, id: &Token, expr: &Expr) -> T;
    fn visit_block(&mut self, statements: &[Stmt]) -> T;
    fn visit_if(&mut self, condition: &Expr, if_block: &Stmt, else_block: &Option<Box<Stmt>>) -> T;
}

impl Stmt {
    pub fn accept<T>(&self, visiter: &mut impl StmtVisitor<T>) -> T {
        match self {
            Stmt::Print(expr) => visiter.visit_print(expr),
            Stmt::Expr(expr) => visiter.visit_expr(expr),
            Stmt::DeclareVar(id, expr) => visiter.visit_declare_var(id, expr),
            Stmt::Block(statements) => visiter.visit_block(statements),
            Stmt::If(expr, if_block, else_block) => visiter.visit_if(expr, if_block, else_block),
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
            Stmt::If(expr, if_block, else_block) => write!(
                f,
                "(if ({}) ({}) ({})",
                expr,
                if_block,
                match else_block {
                    Some(s) => format!("{}", s),
                    None => "null".to_string(),
                }
            ),
        }
    }
}
