use std::collections::HashMap;

use crate::token::Literal;

pub struct Environment {
    values: HashMap<String, Literal>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
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
