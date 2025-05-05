use std::rc::Rc;

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
    DeclareFunc(Token, Rc<Vec<Token>>, Rc<Vec<Stmt>>),
    Return(Expr, u32),
    DeclareClass(
        Token,
        Option<Token>,
        Vec<(Token, Rc<Vec<Token>>, Rc<Vec<Stmt>>)>,
    ),
}

/// A struct that visits `Stmt`
pub trait StmtVisitor<T> {
    fn visit_print(&mut self, stmt: &Expr) -> T;
    fn visit_expr(&mut self, expr: &Expr) -> T;
    fn visit_declare_var(&mut self, id: &Token, expr: &Option<Expr>) -> T;
    fn visit_block(&mut self, statements: &[Stmt]) -> T;
    fn visit_if(&mut self, condition: &Expr, if_block: &Stmt, else_block: &Option<Box<Stmt>>) -> T;
    fn visit_while(&mut self, condition: &Expr, while_block: &Stmt) -> T;
    fn visit_declare_func(
        &mut self,
        id: &Token,
        params: &Rc<Vec<Token>>,
        body: &Rc<Vec<Stmt>>,
    ) -> T;
    fn visit_return(&mut self, expr: &Expr, line: &u32) -> T;
    fn visit_declare_class(
        &mut self,
        id: &Token,
        parent: &Option<Token>,
        methods: &[(Token, Rc<Vec<Token>>, Rc<Vec<Stmt>>)],
    ) -> T;
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
            Stmt::DeclareClass(id, parent, methods) => {
                visiter.visit_declare_class(id, parent, methods)
            }
        }
    }
}
