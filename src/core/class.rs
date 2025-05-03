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
    methods: Rc<HashMap<String, LoxFunction>>,
    arity: usize,
}

impl LoxClass {
    pub fn new(name: String, methods: HashMap<String, LoxFunction>) -> Self {
        let arity = match &methods.get("init") {
            Some(init) => init.arity(),
            None => 0,
        };

        Self {
            name,
            methods: Rc::new(methods),
            arity,
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
        args: Vec<Literal>,
    ) -> Result<Literal, RuntimeError> {
        let instance = Rc::new(RefCell::new(LoxInstance::new(
            self.name.to_string(),
            self.methods.clone(),
        )));

        // Clones the instance to give to the init method
        let bind_instance = instance.clone();

        // We must retrieve the function before calling it because the .call() fn
        // may execute this.x = x assignment expressions which borrow_mut `instance`
        // while it is still borrowed here
        let init = instance
            .borrow()
            .methods
            .get("init")
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

#[derive(Clone, Debug)]
pub struct LoxInstance {
    name: String,
    properties: HashMap<String, Literal>,
    methods: Rc<HashMap<String, LoxFunction>>,
}

impl LoxInstance {
    pub fn new(name: String, methods: Rc<HashMap<String, LoxFunction>>) -> Self {
        Self {
            name,
            properties: HashMap::new(),
            methods,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// ref_self is the same as self, but captured inside a Rc<RefCell<>>
    pub fn get(
        &self,
        token: &Token,
        ref_self: Rc<RefCell<LoxInstance>>,
    ) -> Result<Literal, RuntimeError> {
        match self.properties.get(&token.lexeme) {
            Some(value) => Ok(value.to_owned()),
            None => match self.methods.get(&token.lexeme) {
                Some(func) => Ok(Literal::Callable(Rc::new(func.bind(ref_self)))),
                None => Err(RuntimeError::NameError(token.line, token.lexeme.to_owned())),
            },
        }
    }

    pub fn set(&mut self, token: &Token, value: Literal) {
        self.properties.insert(token.lexeme.to_owned(), value);
    }
}
