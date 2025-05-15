use std::rc::Rc;

use super::Function;

#[derive(Debug)]
pub struct Closure {
    pub function: Rc<Function>,
    pub upvalue_count: u8,
    pub upvalues: Vec<usize>, // Index into VM upvalues array, is this extra level of indirection worth it?
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
