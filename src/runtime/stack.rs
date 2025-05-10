use crate::core::Value;

use super::VM;

impl VM<'_> {
    /// Pushes a new value at the top of the stack
    pub(crate) fn stack_push(&mut self, value: Value) {
        self.stack.push(value);
    }

    /// Removes and returns the elemtn at the top of the stack
    pub(crate) fn stack_pop(&mut self) -> Value {
        self.stack.pop().unwrap_or(Value::nil())
    }

    /// Returns the `i`'th element from the top of the stack
    pub(crate) fn stack_peek(&self, i: usize) -> Value {
        let last = self.stack.len() - 1;
        *self.stack.get(last - i).unwrap_or(&Value::nil())
    }

    /// Returns the `i`th element from the bottom of the stack
    pub(crate) fn stack_get(&self, i: usize) -> Value {
        let fp = self.get_frame().fp;
        *self.stack.get(fp + i).unwrap_or(&Value::nil())
    }

    pub(crate) fn stack_set(&mut self, i: usize, value: Value) {
        let fp = self.get_frame().fp;
        self.stack[fp + i] = value;
    }

    /// Prints a dump of the stack
    pub(crate) fn stack_dump(&self) {
        print!("STACK     ");
        for value in &self.stack {
            print!("[ {} ]", self.format_value(value))
        }
        println!();
    }
}
