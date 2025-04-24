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

    pub fn get_enclosing(&self) -> Option<Rc<RefCell<Environment>>> {
        match &self.enclosing {
            None => None,
            Some(e) => Some(Rc::clone(e)),
        }
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
        }else {
            Err(())
        }
    }
}
