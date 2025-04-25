use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::core::literal::Literal;

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

    pub fn new_enclosed(enclosing: &Rc<RefCell<Self>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            enclosing: Some(Rc::clone(enclosing)),
            values: HashMap::new(),
        }))
    }

    pub fn get_enclosing(&self) -> Option<Rc<RefCell<Environment>>> {
        self.enclosing.as_ref().map(Rc::clone)
    }

    pub fn define(&mut self, id: &String, value: Literal) {
        self.values.insert(id.to_owned(), value);
    }

    pub fn get(&self, id: &String) -> Option<Literal> {
        if let Some(value) = self.values.get(id) {
            return Some(value.clone());
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow().get(id);
        }

        None
    }

    pub fn assign(&mut self, id: &String, value: Literal) -> Result<(), ()> {
        if self.values.contains_key(id) {
            self.values.insert(id.to_owned(), value);
            Ok(())
        } else if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow_mut().assign(id, value);
        } else {
            Err(())
        }
    }
}
