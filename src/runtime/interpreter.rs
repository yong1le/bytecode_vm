use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    ast::{
        expr::{Expr, ExprVisitor},
        stmt::{Stmt, StmtVisitor},
    },
    core::{
        callable::{Clock, LoxFunction},
        class::LoxClass,
        errors::RuntimeError,
        literal::Literal,
        token::{Token, TokenType},
    },
};

use super::environment::Environment;

// A struct that represents an interpreter for evaluating expressions and executing statements.
pub struct Interpreter {
    /// The environment that holds global state.
    globals: Rc<RefCell<Environment>>,
    env: Rc<RefCell<Environment>>,
    locals: HashMap<usize, u32>,
}

impl Interpreter {
    pub fn new() -> Self {
        let env = Environment::new();
        env.borrow_mut()
            .define(&"clock".to_string(), Literal::Callable(Rc::new(Clock)));
        Self {
            env: Rc::clone(&env),
            globals: env,
            locals: HashMap::new(),
        }
    }

    /// Evaluates a single expression and returns its result.
    pub fn evaluate(&mut self, expr: &Expr) -> Result<Literal, RuntimeError> {
        expr.accept(self)
    }

    /// Executes a single statement, storing its side effects into the environment.
    pub fn interpret(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        stmt.accept(self)
    }

    /// Changes the environment to a different one.
    fn set_env(&mut self, env: Rc<RefCell<Environment>>) {
        self.env = env;
    }

    /// Temporary interprets the statements with a different env. On return, changes the env back to
    /// the original one.
    pub fn interpret_with_env(
        &mut self,
        stmts: &[Stmt],
        env: Rc<RefCell<Environment>>,
    ) -> Result<(), RuntimeError> {
        let old_env = Rc::clone(&self.env);
        self.set_env(env);

        let mut return_value: Option<RuntimeError> = None;
        for s in stmts {
            match self.interpret(s) {
                Ok(()) => (),
                Err(e) => {
                    return_value = Some(e);
                    break;
                }
            }
        }

        self.set_env(old_env);

        match return_value {
            None => Ok(()),
            Some(e) => Err(e),
        }
    }

    pub fn resolve(&mut self, token_id: usize, depth: u32) {
        self.locals.insert(token_id, depth);
    }
}

impl ExprVisitor<Result<Literal, RuntimeError>> for Interpreter {
    fn visit_literal(&mut self, literal: &Literal) -> Result<Literal, RuntimeError> {
        Ok(literal.to_owned())
    }

    fn visit_unary(&mut self, operator: &Token, expr: &Expr) -> Result<Literal, RuntimeError> {
        match (&operator.token, self.evaluate(expr)?) {
            (TokenType::Bang, literal) => Ok(Literal::Boolean(!literal.is_truthy())),
            (TokenType::Minus, Literal::Number(num)) => Ok(Literal::Number(-num)),
            (TokenType::Minus, _) => Err(RuntimeError::TypeError(
                operator.line,
                "Operand must be a number.".to_string(),
            )),
            _ => Ok(Literal::Nil),
        }
    }

    fn visit_binary(
        &mut self,
        operator: &Token,
        left: &Expr,
        right: &Expr,
    ) -> Result<Literal, RuntimeError> {
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
                _ => Err(RuntimeError::TypeError(
                    operator.line,
                    "Operand not implemented.".to_string(),
                )),
            },
            // +, ==, !=
            (Literal::String(s1), Literal::String(s2)) => match &operator.token {
                TokenType::Plus => Ok(Literal::String(s1 + s2)),
                TokenType::EqualEqual => Ok(Literal::Boolean(s1 == s2)),
                TokenType::BangEqual => Ok(Literal::Boolean(s1 != s2)),
                _ => Err(RuntimeError::TypeError(
                    operator.line,
                    "Operands must be numbers.".to_string(),
                )),
            },
            (Literal::Boolean(b1), Literal::Boolean(b2)) => match operator.token {
                TokenType::EqualEqual => Ok(Literal::Boolean(b1 == b2)),
                TokenType::BangEqual => Ok(Literal::Boolean(b1 != b2)),
                _ => Err(RuntimeError::TypeError(
                    operator.line,
                    "Operands must be numbers.".to_string(),
                )),
            },
            (Literal::Nil, Literal::Nil) => match operator.token {
                TokenType::EqualEqual => Ok(Literal::Boolean(true)),
                TokenType::BangEqual => Ok(Literal::Boolean(false)),
                _ => Err(RuntimeError::TypeError(
                    operator.line,
                    "Operands must be numbers.".to_string(),
                )),
            },
            _ => match operator.token {
                TokenType::EqualEqual => Ok(Literal::Boolean(false)),
                TokenType::BangEqual => Ok(Literal::Boolean(true)),
                _ => Err(RuntimeError::TypeError(
                    operator.line,
                    "Operands must be numbers.".to_string(),
                )),
            },
        }
    }

    fn visit_grouping(&mut self, expr: &Expr) -> Result<Literal, RuntimeError> {
        self.evaluate(expr)
    }

    fn visit_variable(&mut self, id: &Token) -> Result<Literal, RuntimeError> {
        let result = match self.locals.get(&id.id) {
            Some(depth) => self.env.borrow().get_at(&id.lexeme, depth.to_owned()),
            None => self.globals.borrow().get(&id.lexeme),
        };

        match result {
            Some(var) => Ok(var),
            None => Err(RuntimeError::NameError(id.line, id.lexeme.to_string())),
        }
    }

    fn visit_assignment(&mut self, id: &Token, assignment: &Expr) -> Result<Literal, RuntimeError> {
        let literal = self.evaluate(assignment)?;

        let result = match self.locals.get(&id.id) {
            Some(depth) => {
                self.env
                    .borrow_mut()
                    .assign_at(&id.lexeme, literal.to_owned(), depth.to_owned())
            }
            None => self
                .globals
                .borrow_mut()
                .assign(&id.lexeme, literal.to_owned()),
        };

        match result {
            Ok(()) => Ok(literal),
            Err(()) => Err(RuntimeError::NameError(id.line, id.lexeme.to_owned())),
        }
    }

    fn visit_and(&mut self, left: &Expr, right: &Expr) -> Result<Literal, RuntimeError> {
        let left_eval = self.evaluate(left)?;

        if left_eval.is_truthy() {
            self.evaluate(right)
        } else {
            Ok(left_eval)
        }
    }

    fn visit_or(&mut self, left: &Expr, right: &Expr) -> Result<Literal, RuntimeError> {
        let left_eval = self.evaluate(left)?;

        if left_eval.is_truthy() {
            Ok(left_eval)
        } else {
            self.evaluate(right)
        }
    }

    fn visit_call(
        &mut self,
        callee: &Expr,
        arguments: &[Expr],
        closing: &Token,
    ) -> Result<Literal, RuntimeError> {
        let c = match self.evaluate(callee)? {
            Literal::Callable(c) => c,
            c => {
                return Err(RuntimeError::TypeError(
                    closing.line,
                    format!("{} is not callable.", c),
                ))
            }
        };

        let mut processed_args = Vec::with_capacity(arguments.len());
        for arg in arguments.iter() {
            processed_args.push(self.evaluate(arg)?);
        }

        if processed_args.len() != c.arity() {
            return Err(RuntimeError::TypeError(
                closing.line,
                format!(
                    "Expected {} arguments but got {}.",
                    c.arity(),
                    processed_args.len()
                ),
            ));
        }

        c.call(self, processed_args)
    }
}

impl StmtVisitor<Result<(), RuntimeError>> for Interpreter {
    fn visit_print(&mut self, expr: &Expr) -> Result<(), RuntimeError> {
        match self.evaluate(expr) {
            Ok(literal) => {
                println!("{}", literal.stringify());
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn visit_expr(&mut self, expr: &Expr) -> Result<(), RuntimeError> {
        match self.evaluate(expr) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn visit_declare_var(&mut self, id: &Token, expr: &Option<Expr>) -> Result<(), RuntimeError> {
        let literal = match expr {
            Some(e) => self.evaluate(e)?,
            None => Literal::Nil,
        };

        self.env.borrow_mut().define(&id.lexeme, literal);
        Ok(())
    }

    fn visit_block(&mut self, statements: &[Stmt]) -> Result<(), RuntimeError> {
        let new_env = Environment::new_enclosed(&self.env);
        self.interpret_with_env(statements, new_env)?;
        Ok(())
    }

    fn visit_if(
        &mut self,
        condition: &Expr,
        if_block: &Stmt,
        else_block: &Option<Box<Stmt>>,
    ) -> Result<(), RuntimeError> {
        let condition_bool = self.evaluate(condition)?.is_truthy();

        if condition_bool {
            self.interpret(if_block)
        } else {
            match else_block {
                Some(stmt) => self.interpret(stmt),
                None => Ok(()),
            }
        }
    }

    fn visit_while(&mut self, condition: &Expr, while_block: &Stmt) -> Result<(), RuntimeError> {
        while self.evaluate(condition)?.is_truthy() {
            self.interpret(while_block)?;
        }

        Ok(())
    }

    fn visit_declare_func(
        &mut self,
        id: &Token,
        params: &[Token],
        body: &[Stmt],
    ) -> Result<(), RuntimeError> {
        let function = LoxFunction::new(
            id.to_owned(),
            params.to_owned(),
            body.to_owned(),
            self.env.clone(),
        );

        self.env
            .borrow_mut()
            .define(&id.lexeme, Literal::Callable(Rc::new(function)));

        Ok(())
    }

    fn visit_return(&mut self, expr: &Expr, _: &u32) -> Result<(), RuntimeError> {
        Err(RuntimeError::ReturnValue(self.evaluate(expr)?))
    }

    fn visit_decalre_class(
        &mut self,
        id: &Token,
        methods: &[(Token, Vec<Token>, Vec<Stmt>)],
    ) -> Result<(), RuntimeError> {
        let class = LoxClass::new(id.lexeme.to_string());

        self.env
            .borrow_mut()
            .define(&id.lexeme, Literal::Class(class));

        Ok(())
    }
}
