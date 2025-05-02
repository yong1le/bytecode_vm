use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{callable::LoxCallable, errors::RuntimeError, literal::Literal, token::Token};

#[derive(Clone, Debug)]
pub struct LoxClass {
    name: String,
    methods: Rc<HashMap<String, Rc<dyn LoxCallable>>>,
}

impl LoxClass {
    pub fn new(name: String, methods: HashMap<String, Rc<dyn LoxCallable>>) -> Self {
        Self {
            name,
            methods: Rc::new(methods),
        }
    }
}

impl LoxCallable for LoxClass {
    fn name(&self) -> &str {
        &self.name
    }

    fn call(
        &self,
        _: &mut crate::runtime::interpreter::Interpreter,
        _: Vec<super::literal::Literal>,
    ) -> Result<super::literal::Literal, super::errors::RuntimeError> {
        Ok(Literal::Instance(Rc::new(RefCell::new(LoxInstance::new(
            self.name.to_string(),
            self.methods.clone(),
        )))))
    }

    fn arity(&self) -> usize {
        0
    }
}

#[derive(Clone, Debug)]
pub struct LoxInstance {
    name: String,
    properties: HashMap<String, Literal>,
    methods: Rc<HashMap<String, Rc<dyn LoxCallable>>>,
}

impl LoxInstance {
    pub fn new(name: String, methods: Rc<HashMap<String, Rc<dyn LoxCallable>>>) -> Self {
        Self {
            name,
            properties: HashMap::new(),
            methods,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get(&self, token: &Token) -> Result<Literal, RuntimeError> {
        match self.properties.get(&token.lexeme) {
            Some(value) => Ok(value.to_owned()),
            None => match self.methods.get(&token.lexeme) {
                Some(func) => Ok(Literal::Callable(func.to_owned())),
                None => Err(RuntimeError::NameError(token.line, token.lexeme.to_owned())),
            },
        }
    }

    pub fn set(&mut self, token: &Token, value: Literal) {
        self.properties.insert(token.lexeme.to_owned(), value);
    }
}
