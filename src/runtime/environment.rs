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

    /// Returns the `depth`'th enclosing ancestor of the envionrment
    ///
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
    pub fn define(&mut self, id: String, value: Literal) {
        self.values.insert(id, value);
    }

    /// Gets the value of a variable from the environment. This is equivalent to
    /// `environment.get_at(id, 0)`
    ///
    /// Returns `None` if no value exists.
    pub fn get(&self, id: &str) -> Option<Literal> {
        if let Some(value) = self.values.get(id) {
            return Some(value.own());
        }

        None
    }

    /// Gets the value of a variable at the `depth`'th ancestor of the environment.
    ///
    /// Returns `None` if no value exists.
    pub fn get_at(&self, id: &str, depth: u32) -> Option<Literal> {
        if depth == 0 {
            self.get(id)
        } else {
            self.ancestor(depth).borrow().get(id)
        }
    }

    /// Assigns a value to a variable in the environment. This is equivalent to
    /// `environment.assign_at(id, value, 0)`.
    ///
    /// Returns an error if the variable is not defined in the environment.
    pub fn assign(&mut self, id: &String, value: Literal) -> Result<(), ()> {
        if self.values.contains_key(id) {
            self.values.insert(id.to_owned(), value);
            Ok(())
        } else {
            Err(())
        }
    }

    /// Assigns a value to a variable at defined at the `depth`'th ancestor of
    /// the environment.
    ///
    /// Returns an error if the variable is not defined in the environment.
    pub fn assign_at(&mut self, id: &String, value: Literal, depth: u32) -> Result<(), ()> {
        if depth == 0 {
            self.assign(id, value)
        } else {
            self.ancestor(depth).borrow_mut().assign(id, value)
        }
    }
}
