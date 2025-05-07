use std::collections::HashMap;

use slab::Slab;

use crate::core::value::{Object, Value};

pub type HeapIndex = Value;

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

    pub fn push(&mut self, obj: Object) -> HeapIndex {
        let add_string = match &obj {
            Object::String(s) => {
                if let Some(index) = self.intern_table.get(s) {
                    return Value::object(*index);
                }
                Some(s.clone())
            }
        };

        let index = self.objects.insert(obj);
        if let Some(s) = add_string {
            self.intern_table.insert(s, index);
        }
        Value::object(index)
    }

    pub fn get(&self, value: &HeapIndex) -> Option<&Object> {
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
                }
            )
        }
        println!();
    }
}
