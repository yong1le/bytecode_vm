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
                (TokenType::Bang, Literal::Nil) => Ok(Literal::Boolean(true)),
                (TokenType::Minus, Literal::Number(num)) => Ok(Literal::Number(-num)),
                (TokenType::Minus, _) => Err(EvalError::ValueError("Operand must be a number.")),
                _ => Ok(Literal::Nil),
            },
            Expr::Binary(t, e1, e2) => {
                let l1 = Interpreter::evaluate(*e1)?;
                let l2 = Interpreter::evaluate(*e2)?;

                match (l1, l2) {
                    // -, *, / , <, <=, >, >=, +
                    (Literal::Number(n1), Literal::Number(n2)) => match t.token {
                        TokenType::Plus => Ok(Literal::Number(n1 + n2)),
                        TokenType::Minus => Ok(Literal::Number(n1 - n2)),
                        TokenType::Star => Ok(Literal::Number(n1 * n2)),
                        TokenType::Slash => Ok(Literal::Number(n1 / n2)),
                        TokenType::LessThan => Ok(Literal::Boolean(n1 < n2)),
                        TokenType::LessEqual => Ok(Literal::Boolean(n1 <= n2)),
                        TokenType::GreaterThan => Ok(Literal::Boolean(n1 > n2)),
                        TokenType::GreaterEqual => Ok(Literal::Boolean(n1 >= n2)),
                        _ => Err(EvalError::ValueError("Operand not implemented.")),
                    },
                    // +
                    (Literal::String(s1), Literal::String(s2)) => match t.token {
                        TokenType::Plus => {
                            Ok(Literal::String(s1 + &s2))
                        }
                        _ => Err(EvalError::ValueError("Operands must be numbers.")),
                    },
                    _ => Err(EvalError::ValueError("Operands must be numbers.")),
                }
            }
            Expr::Grouping(e) => Interpreter::evaluate(*e),
        }
    }
}

pub enum EvalError {
    ValueError(&'static str),
}
