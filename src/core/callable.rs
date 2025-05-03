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

use super::{class::LoxInstance, errors::RuntimeError, literal::Literal, token::Token};

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
#[derive(Debug, Clone)]
pub struct LoxFunction {
    name: String,
    params: Rc<Vec<Token>>,
    body: Rc<Vec<Stmt>>,
    closure: Rc<RefCell<Environment>>,
    is_initializer: bool,
}

impl LoxFunction {
    pub fn new(
        name: String,
        params: Rc<Vec<Token>>,
        body: Rc<Vec<Stmt>>,
        closure: Rc<RefCell<Environment>>,
        is_initializer: bool,
    ) -> Self {
        Self {
            name,
            params,
            body,
            closure,
            is_initializer,
        }
    }

    /// Returns a new copy of function that is bound to a class `this`
    pub fn bind(&self, this: Rc<RefCell<LoxInstance>>) -> LoxFunction {
        let env = Environment::new_enclosed(&self.closure);
        env.borrow_mut()
            .define("this".to_string(), Literal::Instance(this));

        LoxFunction::new(
            self.name.clone(),
            self.params.clone(),
            self.body.clone(),
            env,
            self.is_initializer,
        )
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
            env.borrow_mut().define(arg.0.lexeme.to_string(), arg.1);
        }

        match interpreter.interpret_with_env(&self.body, env) {
            Ok(()) => {
                if self.is_initializer {
                    Ok(self.closure.borrow().get("this").unwrap_or(Literal::Nil))
                } else {
                    Ok(Literal::Nil)
                }
            }
            Err(RuntimeError::ReturnValue(l)) => {
                if self.is_initializer {
                    Ok(self.closure.borrow().get("this").unwrap_or(Literal::Nil))
                } else {
                    Ok(l)
                }
            }
            Err(e) => Err(e),
        }
    }

    fn arity(&self) -> usize {
        self.params.len()
    }

    fn name(&self) -> &str {
        &self.name
    }
}
