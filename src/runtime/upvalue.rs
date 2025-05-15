use crate::core::Value;

use super::VM;

#[derive(Debug, Clone, Copy)]
pub enum VMUpvalue {
    Open(usize),   // Index into stack
    Closed(usize), // Index into heap
}

impl VM<'_> {
    pub fn upvalue_get(&self, index: u8) -> Value {
        match self.upvalues[self.frame.closure.upvalues[index as usize]] {
            VMUpvalue::Open(index) => self.stack[index],
            VMUpvalue::Closed(index) => return Value::object(index),
        }
    }
}
