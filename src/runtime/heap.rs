use std::collections::HashMap;

use slab::Slab;

use crate::{core::Value, object::Object};

use super::VM;

pub struct Heap {
    objects: Slab<Object>,
    intern_table: HashMap<String, usize>,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            objects: Slab::new(),
            intern_table: HashMap::new(),
        }
    }

    pub fn push(&mut self, obj: Object) -> Value {
        let add_string = match &obj {
            Object::String(s) => {
                if let Some(index) = self.intern_table.get(s) {
                    return Value::object(*index);
                }
                Some(s.clone())
            }
            _ => None,
        };

        let index = self.objects.insert(obj);
        if let Some(s) = add_string {
            self.intern_table.insert(s, index);
        }
        Value::object(index)
    }

    pub fn get(&self, value: &Value) -> Option<&Object> {
        if !value.is_object() {
            return None;
        }

        self.objects.get(value.as_object())
    }

    pub fn dump(&self) {
        print!("HEAP      ");
        for (_, value) in &self.objects {
            print!(
                " [ {} ]",
                match value {
                    Object::String(s) => s,
                    Object::Function(f) => &f.name,
                    Object::Native(_) => "<native fn>",
                }
            )
        }
        println!();
    }
}

impl VM<'_> {
    /// Returns a mutable reference to the VM's heap
    pub fn heap_mut(&mut self) -> &mut Heap {
        &mut self.heap
    }

    /// Allocates a new entry in the heap, and returns the index
    pub(crate) fn heap_alloc(&mut self, obj: Object) -> Value {
        self.heap.push(obj)
    }

    /// Gets an object on the heap based on the index `value`
    pub(crate) fn heap_get(&self, value: &Value) -> Option<&Object> {
        self.heap.get(value)
    }
}
