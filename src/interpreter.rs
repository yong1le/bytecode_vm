use core::fmt;
use std::{io::{self, Write}, process::exit};

use crate::{
    parser::{
        expr::{Expr, ExprVisitor},
        stmt::{Stmt, StmtVisitor},
    },
    token::{Literal, Token, TokenType},
};

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    /// Evaluates a single expression and returns its result
    pub fn evaluate(&mut self, expr: &Expr) -> Result<Literal, EvalError> {
        expr.accept(self)
    }

    /// Executes a single statement, storing its side effects into the environment
    pub fn interpret(&mut self, stmt: &Stmt) {
        stmt.accept(self)
    }
}

impl ExprVisitor<Result<Literal, EvalError>> for Interpreter {
    fn visit_literal(&mut self, literal: &Literal) -> Result<Literal, EvalError> {
        Ok(literal.to_owned())
    }

    fn visit_unary(&mut self, operator: &Token, expr: &Expr) -> Result<Literal, EvalError> {
        match (&operator.token, self.evaluate(expr)?) {
            (TokenType::Bang, Literal::Boolean(b)) => Ok(Literal::Boolean(!b)),
            (TokenType::Bang, Literal::Number(n)) => Ok(Literal::Boolean(n == 0.0)),
            (TokenType::Bang, Literal::String(s)) => Ok(Literal::Boolean(s.is_empty())),
            (TokenType::Bang, Literal::Nil) => Ok(Literal::Boolean(true)),
            (TokenType::Minus, Literal::Number(num)) => Ok(Literal::Number(-num)),
            (TokenType::Minus, _) => Err(EvalError::ValueError("Operand must be a number.")),
            _ => Ok(Literal::Nil),
        }
    }

    fn visit_binary(
        &mut self,
        operator: &Token,
        left: &Expr,
        right: &Expr,
    ) -> Result<Literal, EvalError> {
        let l1 = self.evaluate(left)?;
        let l2 = self.evaluate(right)?;

        match (l1, l2) {
            // -, *, / , <, <=, >, >=, +, ==, !=
            (Literal::Number(n1), Literal::Number(n2)) => match &operator.token {
                TokenType::Plus => Ok(Literal::Number(n1 + n2)),
                TokenType::Minus => Ok(Literal::Number(n1 - n2)),
                TokenType::Star => Ok(Literal::Number(n1 * n2)),
                TokenType::Slash => Ok(Literal::Number(n1 / n2)),
                TokenType::LessThan => Ok(Literal::Boolean(n1 < n2)),
                TokenType::LessEqual => Ok(Literal::Boolean(n1 <= n2)),
                TokenType::GreaterThan => Ok(Literal::Boolean(n1 > n2)),
                TokenType::GreaterEqual => Ok(Literal::Boolean(n1 >= n2)),
                TokenType::EqualEqual => Ok(Literal::Boolean(n1 == n2)),
                TokenType::BangEqual => Ok(Literal::Boolean(n1 != n2)),
                _ => Err(EvalError::ValueError("Operand not implemented.")),
            },
            // +, ==, !=
            (Literal::String(s1), Literal::String(s2)) => match &operator.token {
                TokenType::Plus => Ok(Literal::String(s1 + s2)),
                TokenType::EqualEqual => Ok(Literal::Boolean(s1 == s2)),
                TokenType::BangEqual => Ok(Literal::Boolean(s1 != s2)),
                _ => Err(EvalError::ValueError("Operands must be numbers.")),
            },
            (Literal::Boolean(b1), Literal::Boolean(b2)) => match operator.token {
                TokenType::EqualEqual => Ok(Literal::Boolean(b1 == b2)),
                TokenType::BangEqual => Ok(Literal::Boolean(b1 != b2)),
                _ => Err(EvalError::ValueError("Operands must be numbers.")),
            },
            (Literal::Nil, Literal::Nil) => match operator.token {
                TokenType::EqualEqual => Ok(Literal::Boolean(true)),
                TokenType::BangEqual => Ok(Literal::Boolean(false)),
                _ => Err(EvalError::ValueError("Operands must be numbers.")),
            },
            _ => match operator.token {
                TokenType::EqualEqual => Ok(Literal::Boolean(false)),
                TokenType::BangEqual => Ok(Literal::Boolean(true)),
                _ => Err(EvalError::ValueError("Operands must be numbers.")),
            },
        }
    }

    fn visit_grouping(&mut self, expr: &Expr) -> Result<Literal, EvalError> {
        self.evaluate(expr)
    }
}

impl StmtVisitor for Interpreter {
    fn visit_print(&mut self, expr: &Expr) {
        match self.evaluate(expr) {
            Ok(literal) => {
                println!("{}", literal.stringify());
            }
            Err(e) => {
                writeln!(io::stderr(), "{}", e);
                exit(70);
            }
        };
    }

    fn visit_expr(&mut self, expr: &Expr) {
        match self.evaluate(expr) {
            Ok(literal) => (),
            Err(e) => {
                writeln!(io::stderr(), "{}", e);
                exit(70);
            }
        };
    }
}

pub enum EvalError {
    ValueError(&'static str),
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::ValueError(s) => write!(f, "{}", s),
        }
    }
}
