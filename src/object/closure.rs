use std::rc::Rc;

use super::{Function, VMUpvalue};

#[derive(Debug)]
pub struct Closure {
    pub function: Rc<Function>,
    pub upvalue_count: u8,
    pub upvalues: Vec<VMUpvalue>,
}

impl Closure {
    pub fn new(function: Rc<Function>, upvalue_count: u8) -> Self {
        Self {
            function,
            upvalue_count,
            upvalues: Vec::with_capacity(upvalue_count as usize),
        }
    }
}
