use crate::core::token::Token;

/// Enum to represent different types of expressions in the AST.
#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Token),
    Unary(Token, Box<Expr>),
    Binary(Token, Box<Expr>, Box<Expr>),
    Grouping(Box<Expr>),
    Variable(Token),
    Assign(Token, Box<Expr>),
    And(Token, Box<Expr>, Box<Expr>),
    Or(Token, Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>, Token),
    Get(Box<Expr>, Token),
    Set(Box<Expr>, Token, Box<Expr>),
    This(Token),
    Super(Token, Token),
}

/// A struct that visits `Expr`
pub trait ExprVisitor<T> {
    fn visit_literal(&mut self, token: &Token) -> T;
    fn visit_unary(&mut self, operator: &Token, expr: &Expr) -> T;
    fn visit_binary(&mut self, operator: &Token, left: &Expr, right: &Expr) -> T;
    fn visit_grouping(&mut self, expr: &Expr) -> T;
    fn visit_variable(&mut self, id: &Token) -> T;
    fn visit_assignment(&mut self, id: &Token, assignment: &Expr) -> T;
    fn visit_and(&mut self, token: &Token, left: &Expr, right: &Expr) -> T;
    fn visit_or(&mut self, token: &Token, left: &Expr, right: &Expr) -> T;
    fn visit_call(&mut self, callee: &Expr, arguments: &[Expr], closing: &Token) -> T;
    fn visit_get(&mut self, obj: &Expr, prop: &Token) -> T;
    fn visit_set(&mut self, obj: &Expr, prop: &Token, value: &Expr) -> T;
    fn visit_this(&mut self, token: &Token) -> T;
    fn visit_super(&mut self, super_token: &Token, prop: &Token) -> T;
}

impl Expr {
    pub fn accept<T>(&self, visitor: &mut impl ExprVisitor<T>) -> T {
        match self {
            Expr::Literal(token) => visitor.visit_literal(token),
            Expr::Unary(op, expr) => visitor.visit_unary(op, expr),
            Expr::Binary(op, left, right) => visitor.visit_binary(op, left, right),
            Expr::Grouping(expr) => visitor.visit_grouping(expr),
            Expr::Variable(id) => visitor.visit_variable(id),
            Expr::Assign(id, assignment) => visitor.visit_assignment(id, assignment),
            Expr::And(token, left, right) => visitor.visit_and(token, left, right),
            Expr::Or(token, left, right) => visitor.visit_or(token, left, right),
            Expr::Call(callee, arguments, closing) => {
                visitor.visit_call(callee, arguments, closing)
            }
            Expr::Get(obj, prop) => visitor.visit_get(obj, prop),
            Expr::Set(obj, prop, value) => visitor.visit_set(obj, prop, value),
            Expr::This(token) => visitor.visit_this(token),
            Expr::Super(super_token, prop) => visitor.visit_super(super_token, prop),
        }
    }
}
