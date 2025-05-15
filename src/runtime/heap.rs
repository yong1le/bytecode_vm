use std::rc::Rc;

use rustc_hash::FxHashMap;
use slab::Slab;

use crate::{core::Value, object::Object};

use super::VM;

pub struct Heap {
    objects: Slab<Object>,
    intern_table: FxHashMap<Rc<str>, usize>,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            objects: Slab::new(),
            intern_table: FxHashMap::default(),
        }
    }

    /// Pushes an object into the heap and return its index as a Value.
    /// Strings should use [`Heap::push_str`]
    pub fn push(&mut self, obj: Object) -> Value {
        let index = self.objects.insert(obj);
        Value::object(index)
    }

    pub fn push_str(&mut self, s: String) -> Value {
        let string: Rc<str> = Rc::from(s);
        if let Some(index) = self.intern_table.get(&string) {
            Value::object(*index)
        } else {
            let index = self.objects.insert(Object::String(string.clone()));
            self.intern_table.insert(string, index);
            Value::object(index)
        }
    }

    pub fn get(&self, value: &Value) -> Option<&Object> {
        if !value.is_object() {
            return None;
        }

        self.objects.get(value.as_object())
    }

    pub(crate) fn set(&mut self, index: usize, value: Value) {
        match &self.objects[index] {
            Object::UpValue(v) => {
                *v.borrow_mut() = value;
            }
            _ => {
                panic!("trying to mutate immutable value")
            }
        }
    }

    pub fn dump(&self) {
        eprint!("HEAP     ");
        for (_, value) in &self.objects {
            eprint!(" [ {} ]", self.format_value(value))
        }
        eprintln!();
    }

    pub fn format_value(&self, value: &Object) -> String {
        match value {
            Object::String(s) => s.to_string(),
            Object::Function(f) => format!("<fn {}>", f.name),
            Object::Native(f) => format!("<fn {}>", f.name()),
            Object::Closure(f) => format!("<closure {}>", f.function.name),
            Object::UpValue(v) => match v.borrow() {
                o if o.is_object() => self.format_value(self.get(&o).unwrap()),
                a => format!("{:?}", a),
            },
        }
    }
}

impl VM<'_> {
    /// Returns a mutable reference to the VM's heap
    pub fn heap_mut(&mut self) -> &mut Heap {
        &mut self.heap
    }

    /// Gets an object on the heap based on the index `value`
    pub(crate) fn heap_get(&self, value: &Value) -> Option<&Object> {
        self.heap.get(value)
    }
}
