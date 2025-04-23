use core::fmt;

use super::expr::Expr;

pub enum Stmt {
    Print(Expr),
    Expr(Expr),
}

pub trait StmtVisitor {
    fn visit_print(&mut self, stmt: &Expr);
    fn visit_expr(&mut self, expr: &Expr);
}

impl Stmt {
    pub fn accept(&self, visiter: &mut impl StmtVisitor) {
        match self {
            Stmt::Print(expr) => visiter.visit_print(expr),
            Stmt::Expr(expr) => visiter.visit_expr(expr),
        }
    }
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Expr(e) => write!(f, "{}", e),
            Stmt::Print(e) => write!(f, "(print {})", e)
        }
    }
}