use crate::{
    parser::Expr,
    token::{Literal, TokenType},
};

pub struct Interpreter {}

impl Interpreter {
    pub fn evaluate(expr: Expr) -> Result<Literal, EvalError> {
        match expr {
            Expr::Literal(l) => Ok(l),
            Expr::Unary(t, e) => match (t.token, Interpreter::evaluate(*e)?) {
                (TokenType::Bang, Literal::Boolean(b)) => Ok(Literal::Boolean(!b)),
                (TokenType::Bang, Literal::Number(n)) => Ok(Literal::Boolean(n == 0.0)),
                (TokenType::Bang, Literal::String(s)) => Ok(Literal::Boolean(s.is_empty())),
                (TokenType::Bang, Literal::None) => Ok(Literal::Boolean(true)),
                (TokenType::Minus, Literal::Number(num)) => Ok(Literal::Number(-num)),
                (TokenType::Minus, _) => Err(EvalError::ValueError("Operand must be a number.")),
                _ => Ok(Literal::None),
            },
            Expr::Binary(t, e1, e2) => Err(EvalError::ValueError("Not impl")),
            Expr::Grouping(e) =>{
              Interpreter::evaluate(*e)
            },
        }
    }
}

pub enum EvalError {
    ValueError(&'static str),
}
