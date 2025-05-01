use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::core::literal::Literal;

/// An environment that holds variables and their values.
#[derive(Debug, Clone)]
pub struct Environment {
    /// The enclosing environment, if any.
    enclosing: Option<Rc<RefCell<Environment>>>,
    /// The variables and their values in this environment.
    values: HashMap<String, Literal>,
}

impl Environment {
    /// Returns a new environment.
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            enclosing: None,
            values: HashMap::new(),
        }))
    }

    /// Returns a new environment that is enclosed within the given environment.
    pub fn new_enclosed(enclosing: &Rc<RefCell<Self>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            enclosing: Some(Rc::clone(enclosing)),
            values: HashMap::new(),
        }))
    }

    /// Condition: depth >= 1
    fn ancestor(&self, depth: u32) -> Rc<RefCell<Environment>> {
        match &self.enclosing {
            Some(enclosing) => {
                if depth == 1 {
                    Rc::clone(enclosing)
                } else {
                    enclosing.borrow().ancestor(depth - 1)
                }
            }
            None => panic!(
                "Attempted to access ancestor environment at depth {}, but it doesn't exist. [This should never happen]",
                depth
            ),
        }
    }

    /// Defines a variable in the environment.
    pub fn define(&mut self, id: &String, value: Literal) {
        self.values.insert(id.to_owned(), value);
    }

    /// Gets the value of a variable from the environment, or any enclosing environment.
    pub fn get(&self, id: &String) -> Option<Literal> {
        if let Some(value) = self.values.get(id) {
            return Some(value.clone());
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow().get(id);
        }

        None
    }

    pub fn get_at(&self, id: &String, depth: u32) -> Option<Literal> {
        if depth == 0 {
            self.get(id)
        } else {
            self.ancestor(depth).borrow().get(id)
        }
    }

    /// Assigns a value to a variable in the environment, or any enclosing environment.
    /// If no variable is found, it returns an error.
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

    pub fn assign_at(&mut self, id: &String, value: Literal, depth: u32) -> Result<(), ()> {
        if depth == 0 {
            self.assign(id, value)
        } else {
            self.ancestor(depth).borrow_mut().assign(id, value)
        }
    }
}
