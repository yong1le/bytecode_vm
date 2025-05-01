use core::fmt;
use std::{
    cell::RefCell,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    ast::stmt::Stmt,
    runtime::{environment::Environment, interpreter::Interpreter},
};

use super::{errors::RuntimeError, literal::Literal, token::Token};

pub trait LoxCallable: fmt::Debug {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Literal>,
    ) -> Result<Literal, RuntimeError>;
    fn arity(&self) -> usize;
    fn name(&self) -> &str;
}

// Native Functions
#[derive(Debug)]
pub struct Clock;
impl LoxCallable for Clock {
    fn call(&self, _: &mut Interpreter, _: Vec<Literal>) -> Result<Literal, RuntimeError> {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards.");

        Ok(Literal::Number(time.as_secs_f64().trunc()))
    }

    fn arity(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "clock"
    }
}

// User Functions
#[derive(Debug)]
pub struct LoxFunction {
    id: Token,
    params: Vec<Token>,
    body: Vec<Stmt>,
    closure: Rc<RefCell<Environment>>,
}

impl LoxFunction {
    pub fn new(
        id: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Environment>>,
    ) -> Self {
        Self {
            id,
            params,
            body,
            closure,
        }
    }
}

impl LoxCallable for LoxFunction {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Literal>,
    ) -> Result<Literal, RuntimeError> {
        let env = Environment::new_enclosed(&self.closure);

        for arg in self.params.iter().zip(arguments) {
            env.borrow_mut().define(&arg.0.lexeme, arg.1);
        }

        match interpreter.interpret_with_env(&self.body, env) {
            Ok(()) => Ok(Literal::Nil),
            Err(RuntimeError::ReturnValue(l)) => Ok(l),
            Err(e) => Err(e),
        }
    }

    fn arity(&self) -> usize {
        self.params.len()
    }

    fn name(&self) -> &str {
        &self.id.lexeme
    }
}
