use std::rc::Rc;

use super::Function;

#[derive(Debug)]
pub struct Closure {
    pub function: Rc<Function>,
}

impl Closure {
    pub fn new(function: Rc<Function>) -> Self {
        Self { function }
    }
}
