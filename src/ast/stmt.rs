use core::fmt;

use crate::core::token::Token;

use super::expr::Expr;

/// Enum to represent different types of statements in the AST.
#[derive(Debug, Clone)]
pub enum Stmt {
    Print(Expr),
    Expr(Expr),
    DeclareVar(Token, Option<Expr>),
    Block(Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    DeclareFunc(Token, Vec<Token>, Vec<Stmt>),
    Return(Expr, u32),
    DeclareClass(Token, Vec<(Token, Vec<Token>, Vec<Stmt>)>),
}

/// A struct that visits `Stmt`
pub trait StmtVisitor<T> {
    fn visit_print(&mut self, stmt: &Expr) -> T;
    fn visit_expr(&mut self, expr: &Expr) -> T;
    fn visit_declare_var(&mut self, id: &Token, expr: &Option<Expr>) -> T;
    fn visit_block(&mut self, statements: &[Stmt]) -> T;
    fn visit_if(&mut self, condition: &Expr, if_block: &Stmt, else_block: &Option<Box<Stmt>>) -> T;
    fn visit_while(&mut self, condition: &Expr, while_block: &Stmt) -> T;
    fn visit_declare_func(&mut self, id: &Token, params: &[Token], body: &[Stmt]) -> T;
    fn visit_return(&mut self, expr: &Expr, line: &u32) -> T;
    fn visit_declare_class(&mut self, id: &Token, methods: &[(Token, Vec<Token>, Vec<Stmt>)]) -> T;
}

impl Stmt {
    pub fn accept<T>(&self, visiter: &mut impl StmtVisitor<T>) -> T {
        match self {
            Stmt::Print(expr) => visiter.visit_print(expr),
            Stmt::Expr(expr) => visiter.visit_expr(expr),
            Stmt::DeclareVar(id, expr) => visiter.visit_declare_var(id, expr),
            Stmt::Block(statements) => visiter.visit_block(statements),
            Stmt::If(expr, if_block, else_block) => visiter.visit_if(expr, if_block, else_block),
            Stmt::While(expr, stmt) => visiter.visit_while(expr, stmt),
            Stmt::DeclareFunc(id, params, body) => visiter.visit_declare_func(id, params, body),
            Stmt::Return(expr, line) => visiter.visit_return(expr, line),
            Stmt::DeclareClass(id, methods) => visiter.visit_declare_class(id, methods),
        }
    }
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Expr(e) => write!(f, "{}", e),
            Stmt::Print(e) => write!(f, "(print {})", e),
            Stmt::DeclareVar(id, expr) => write!(
                f,
                "(var {} ({}))",
                id.lexeme,
                match expr {
                    Some(e) => format!("{e}"),
                    None => "null".to_string(),
                }
            ),
            Stmt::Block(stmts) => {
                writeln!(f, "(block [")?;
                for stmt in stmts {
                    writeln!(f, "  {}", stmt)?;
                }
                write!(f, "])")
            }
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
            Stmt::While(expr, while_block) => write!(f, "(while ({}) ({}))", expr, while_block),
            Stmt::DeclareClass(id, _) => write!(f, "{}", id.lexeme),
            _ => todo!(),
        }
    }
}
