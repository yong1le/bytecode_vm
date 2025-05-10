use crate::core::token::Token;

use super::expr::Expr;

/// Enum to represent different types of statements in the AST.
#[derive(Debug, Clone)]
pub enum Stmt {
    Print(Token, Expr),
    Expr(Token, Expr),
    DeclareVar(Token, Option<Expr>),
    Block(Vec<Stmt>),
    If(Token, Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Token, Expr, Box<Stmt>),
    DeclareFunc(Token, Vec<Token>, Vec<Stmt>),
    Return(Token, Expr),
    DeclareClass(Token, Option<Token>, Vec<(Token, Vec<Token>, Vec<Stmt>)>),
}

/// A struct that visits `Stmt`
pub trait StmtVisitor<T> {
    fn visit_print(&mut self, token: Token, stmt: Expr) -> T;
    fn visit_expr(&mut self, token: Token, expr: Expr) -> T;
    fn visit_declare_var(&mut self, id: Token, expr: Option<Expr>) -> T;
    fn visit_block(&mut self, statements: Vec<Stmt>) -> T;
    fn visit_if(
        &mut self,
        token: Token,
        condition: Expr,
        if_block: Stmt,
        else_block: Option<Box<Stmt>>,
    ) -> T;
    fn visit_while(&mut self, token: Token, condition: Expr, while_block: Stmt) -> T;
    fn visit_declare_func(&mut self, id: Token, params: Vec<Token>, body: Vec<Stmt>) -> T;
    fn visit_return(&mut self, token: Token, expr: Expr) -> T;
    fn visit_declare_class(
        &mut self,
        id: Token,
        parent: Option<Token>,
        methods: Vec<(Token, Vec<Token>, Vec<Stmt>)>,
    ) -> T;
}

impl Stmt {
    pub fn accept<T>(self, visiter: &mut impl StmtVisitor<T>) -> T {
        match self {
            Stmt::Print(token, expr) => visiter.visit_print(token, expr),
            Stmt::Expr(token, expr) => visiter.visit_expr(token, expr),
            Stmt::DeclareVar(id, expr) => visiter.visit_declare_var(id, expr),
            Stmt::Block(statements) => visiter.visit_block(statements),
            Stmt::If(token, expr, if_block, else_block) => {
                visiter.visit_if(token, expr, *if_block, else_block)
            }
            Stmt::While(token, expr, stmt) => visiter.visit_while(token, expr, *stmt),
            Stmt::DeclareFunc(id, params, body) => visiter.visit_declare_func(id, params, body),
            Stmt::Return(token, expr) => visiter.visit_return(token, expr),
            Stmt::DeclareClass(id, parent, methods) => {
                visiter.visit_declare_class(id, parent, methods)
            }
        }
    }
}
