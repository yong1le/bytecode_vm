use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{
    callable::{LoxCallable, LoxFunction},
    token::Token,
    value::Value,
};

/// User defined classes
#[derive(Clone, Debug)]
pub struct LoxClass {
    name: String,
    parent: Option<Rc<LoxClass>>,
    methods: HashMap<String, LoxFunction>,
    arity: usize,
}

impl LoxClass {
    pub fn new(
        name: String,
        parent: Option<Rc<LoxClass>>,
        methods: HashMap<String, LoxFunction>,
    ) -> Rc<Self> {
        let arity = match &methods.get("init") {
            Some(init) => init.arity(),
            None => 0,
        };

        Rc::new(Self {
            name,
            parent,
            methods,
            arity,
        })
    }

    /// Finds a method that was defined in this class. If not method was found, look up
    /// the ancestory tree.
    pub fn find_method(&self, method: &str) -> Option<&LoxFunction> {
        match self.methods.get(method) {
            Some(func) => Some(func),
            None => {
                if let Some(parent) = &self.parent {
                    parent.find_method(method)
                } else {
                    None
                }
            }
        }
    }
}

impl LoxCallable for LoxClass {
    fn name(&self) -> &str {
        &self.name
    }

    fn call(&self, mut args: Vec<Value>) {
        todo!()
InterpretError::UnImplemented    }

    fn arity(&self) -> usize {
        self.arity
    }
}

#[derive(Debug, Clone)]
pub struct LoxInstance {
    class: Rc<LoxClass>,
    properties: HashMap<String, Value>,
}

impl LoxInstance {
    pub fn new(class: Rc<LoxClass>) -> Self {
        Self {
            properties: HashMap::new(),
            class,
        }
    }

    pub fn name(&self) -> &str {
        &self.class.name
    }

    /// ref_self is the same as self, but captured inside a Rc<RefCell<>>
    pub fn get(&self, token: &Token, self_ref: Rc<RefCell<LoxInstance>>) {
        todo!()
InterpretError::UnImplemented    }

    pub fn set(&mut self, token: &Token, value: Value) {
        todo!()
InterpretError::UnImplemented    }
}
