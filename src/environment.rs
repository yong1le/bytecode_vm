use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::token::Literal;

pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Literal>,
}

impl Environment {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            enclosing: None,
            values: HashMap::new(),
        }))
    }

    pub fn block(enclosing: &Rc<RefCell<Self>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            enclosing: Some(Rc::clone(enclosing)),
            values: HashMap::new(),
        }))
    }

    pub fn define(&mut self, id: &String, value: Literal) {
        self.values.insert(id.to_owned(), value);
    }

    pub fn get(&self, id: &String) -> Option<&Literal> {
        self.values.get(id)
    }

    pub fn assign(&mut self, id: &String, value: Literal) -> Result<(), ()>{
        if self.values.contains_key(id) {
            self.values.insert(id.to_owned(), value);
            Ok(())
        } else {
            Err(())
        }
    }
}
