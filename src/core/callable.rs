use core::fmt;
use std::{
    cell::RefCell,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::ast::stmt::Stmt;

use super::{class::LoxInstance, token::Token, value::Value};

/// A trait that represents a callable object in Lox
pub trait LoxCallable: fmt::Debug {
    fn call(&self, arguments: Vec<Value>);
    fn arity(&self) -> usize;
    fn name(&self) -> &str;
}

// Native Functions
#[derive(Debug)]
pub struct Clock;
impl LoxCallable for Clock {
    fn call(&self, _: Vec<Value>) {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards.");

        InterpretError::UnImplemented
    }

    fn arity(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "clock"
    }
}

/// User Functions
#[derive(Debug, Clone)]
pub struct LoxFunction {
    name: String,
    params: Rc<Vec<Token>>,
    body: Rc<Vec<Stmt>>,
    is_initializer: bool,
}

impl LoxFunction {
    pub fn new(
        name: String,
        params: Rc<Vec<Token>>,
        body: Rc<Vec<Stmt>>,
        is_initializer: bool,
    ) -> Self {
        Self {
            name,
            params,
            body,
            is_initializer,
        }
    }
}

impl LoxCallable for LoxFunction {
    fn call(&self, arguments: Vec<Value>) {
        InterpretError::UnImplemented
    }

    fn arity(&self) -> usize {
        self.params.len()
    }

    fn name(&self) -> &str {
        &self.name
    }
}
