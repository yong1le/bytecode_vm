use crate::{
    ast::{
        expr::{Expr, ExprVisitor},
        stmt::{Stmt, StmtVisitor},
    },
    environment::Environment,
    errors::EvalError,
    token::{Literal, Token, TokenType},
};

pub struct Interpreter {
    env: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
        }
    }

    /// Evaluates a single expression and returns its result
    pub fn evaluate(&mut self, expr: &Expr) -> Result<Literal, EvalError> {
        expr.accept(self)
    }

    /// Executes a single statement, storing its side effects into the environment
    pub fn interpret(&mut self, stmt: &Stmt) -> Result<(), EvalError> {
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
            (TokenType::Minus, _) => Err(EvalError::TypeError(operator.line, "Operand must be a number.")),
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
                _ => Err(EvalError::TypeError(operator.line, "Operand not implemented.")),
            },
            // +, ==, !=
            (Literal::String(s1), Literal::String(s2)) => match &operator.token {
                TokenType::Plus => Ok(Literal::String(s1 + s2)),
                TokenType::EqualEqual => Ok(Literal::Boolean(s1 == s2)),
                TokenType::BangEqual => Ok(Literal::Boolean(s1 != s2)),
                _ => Err(EvalError::TypeError(operator.line, "Operands must be numbers.")),
            },
            (Literal::Boolean(b1), Literal::Boolean(b2)) => match operator.token {
                TokenType::EqualEqual => Ok(Literal::Boolean(b1 == b2)),
                TokenType::BangEqual => Ok(Literal::Boolean(b1 != b2)),
                _ => Err(EvalError::TypeError(operator.line, "Operands must be numbers.")),
            },
            (Literal::Nil, Literal::Nil) => match operator.token {
                TokenType::EqualEqual => Ok(Literal::Boolean(true)),
                TokenType::BangEqual => Ok(Literal::Boolean(false)),
                _ => Err(EvalError::TypeError(operator.line, "Operands must be numbers.")),
            },
            _ => match operator.token {
                TokenType::EqualEqual => Ok(Literal::Boolean(false)),
                TokenType::BangEqual => Ok(Literal::Boolean(true)),
                _ => Err(EvalError::TypeError(operator.line, "Operands must be numbers.")),
            },
        }
    }

    fn visit_grouping(&mut self, expr: &Expr) -> Result<Literal, EvalError> {
        self.evaluate(expr)
    }

    fn visit_variable(&mut self, id: &Token) -> Result<Literal, EvalError> {
        match self.env.get(&id.lexeme) {
            Some(var) => Ok(var.to_owned()),
            None => Err(EvalError::NameError(id.line, id.to_string())),
        }
    }

    fn visit_assignment(&mut self, id: &Token, assignment: &Expr) -> Result<Literal, EvalError> {
        let literal = self.evaluate(assignment);
        match literal {
            Ok(l) => match self.env.assign(&id.lexeme, l.to_owned()) {
                Ok(()) => Ok(l.to_owned()),
                Err(()) => Err(EvalError::NameError(id.line, id.lexeme.to_owned())),
            },
            Err(e) => Err(e),
        }
    }
}

impl StmtVisitor<Result<(), EvalError>> for Interpreter {
    fn visit_print(&mut self, expr: &Expr) -> Result<(), EvalError> {
        match self.evaluate(expr) {
            Ok(literal) => {
                println!("{}", literal.stringify());
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn visit_expr(&mut self, expr: &Expr) -> Result<(), EvalError> {
        match self.evaluate(expr) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn visit_declare_var(&mut self, id: &Token, expr: &Expr) -> Result<(), EvalError> {
        let literal = self.evaluate(expr);
        match literal {
            Ok(l) => {
                self.env.define(&id.lexeme, l);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
