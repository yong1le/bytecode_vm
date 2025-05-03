use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::runtime::interpreter::Interpreter;

use super::{
    callable::{LoxCallable, LoxFunction},
    errors::RuntimeError,
    literal::Literal,
    token::Token,
};

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

    fn find_method(&self, method: &str) -> Option<&LoxFunction> {
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

    fn call(
        &self,
        interpreter: &mut Interpreter,
        mut args: Vec<Literal>,
    ) -> Result<Literal, RuntimeError> {
        // The last argument will always be an Rc reference to self
        let instance = match args.pop() {
            Some(Literal::Class(self_ref)) => Rc::new(RefCell::new(LoxInstance::new(self_ref))),
            _ => panic!("LoxClass() called without Rc<LoxClass> as the last argument"),
        };

        // Clones the instance to give to the init method
        let bind_instance = instance.clone();

        // We must retrieve the function before calling it because the .call() fn
        // may execute this.x = x assignment expressions which borrow_mut `instance`
        // while it is still borrowed here
        let init = self
            .find_method("init")
            .map(|init| init.bind(bind_instance));

        if let Some(init) = init {
            init.call(interpreter, args)?;
        };

        Ok(Literal::Instance(instance))
    }

    fn arity(&self) -> usize {
        self.arity
    }
}

#[derive(Debug, Clone)]
pub struct LoxInstance {
    class: Rc<LoxClass>,
    properties: HashMap<String, Literal>,
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
    pub fn get(
        &self,
        token: &Token,
        self_ref: Rc<RefCell<LoxInstance>>,
    ) -> Result<Literal, RuntimeError> {
        match self.properties.get(&token.lexeme) {
            Some(value) => Ok(value.to_owned()),
            None => {
                if let Some(func) = self.class.find_method(&token.lexeme) {
                    Ok(Literal::Callable(Rc::new(func.bind(self_ref))))
                } else {
                    Err(RuntimeError::NameError(token.line, token.lexeme.to_owned()))
                }
            }
        }
    }

    pub fn set(&mut self, token: &Token, value: Literal) {
        self.properties.insert(token.lexeme.to_owned(), value);
    }
}
