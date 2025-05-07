use std::collections::HashMap;

use crate::{
    core::{
        errors::{CompileError, InterpretError, RuntimeError},
        value::Object,
    },
    heap::{Heap, HeapIndex},
    opcode::OpCode,
    Chunk, Value,
};

pub struct VM {
    ip: usize,
    stack: Vec<Value>,
    heap: Heap,
    globals: HashMap<String, Value>,
    chunk: Chunk,
}

type Return = Result<(), InterpretError>;

impl VM {
    const STACK_MAX: usize = 256;
    pub fn new() -> Self {
        VM {
            ip: 0,
            stack: Vec::with_capacity(Self::STACK_MAX),
            heap: Heap::new(),
            globals: HashMap::new(),
            chunk: Chunk::new(),
        }
    }

    /// Returns a mutable reference to the VM's heap
    pub fn heap(&mut self) -> &mut Heap {
        &mut self.heap
    }

    /// Pushes a new value at the back of the stack
    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    /// Removes and returns the last value on the stack
    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap_or(Value::nil())
    }

    /// Allocates a new entry in the heap, and returns the index
    fn alloc(&mut self, obj: Object) -> HeapIndex {
        self.heap.push(obj)
    }

    /// Gets an object on the heap based on the index `value`
    fn get_obj(&self, value: &HeapIndex) -> Option<&Object> {
        self.heap.get(value)
    }

    /// Prints a dump of the stack
    fn stack_dump(&self) {
        print!("STACK     ");
        for value in &self.stack {
            print!("[ {} ]", self.format_value(value))
        }
        println!();
    }

    fn format_value(&self, value: &Value) -> String {
        if value.is_object() {
            match self.get_obj(value) {
                Some(Object::String(s)) => s.to_string(),
                None => "nil".to_string(),
            }
        } else if value.is_number() {
            format!("{}", value.as_number())
        } else if value.is_boolean() {
            format!("{}", value.as_boolean())
        } else if value.is_nil() {
            "nil".to_string()
        } else {
            panic!("Inavlid bit sequence for value");
        }
    }

    pub fn run(&mut self, chunk: Chunk) -> Return {
        self.ip = 0;
        self.chunk = chunk;

        while self.ip < self.chunk.code.len() {
            let op = self.chunk.code[self.ip];

            #[cfg(debug_assertions)]
            {
                self.stack_dump();
                self.heap.dump();
                self.chunk.disassemble_instruction(self.ip);
            }

            match OpCode::try_from(op) {
                Ok(OpCode::Constant) => self.run_constant(false)?,
                Ok(OpCode::ConstantLong) => self.run_constant(true)?,
                Ok(OpCode::Negate) => self.run_negate()?,
                Ok(OpCode::Not) => self.run_not()?,
                Ok(OpCode::Add) => self.run_add()?,
                Ok(OpCode::Subtract) => self.run_numeric_binary(OpCode::Subtract)?,
                Ok(OpCode::Multiply) => self.run_numeric_binary(OpCode::Multiply)?,
                Ok(OpCode::Divide) => self.run_numeric_binary(OpCode::Divide)?,
                Ok(OpCode::Equal) => self.run_equals()?,
                Ok(OpCode::NotEqual) => self.run_numeric_binary(OpCode::NotEqual)?,
                Ok(OpCode::LessEqual) => self.run_numeric_binary(OpCode::LessEqual)?,
                Ok(OpCode::LessThan) => self.run_numeric_binary(OpCode::LessThan)?,
                Ok(OpCode::GreaterThan) => self.run_numeric_binary(OpCode::GreaterThan)?,
                Ok(OpCode::GreaterEqual) => self.run_numeric_binary(OpCode::GreaterEqual)?,
                Ok(OpCode::Print) => {
                    let constant = self.pop();
                    println!("{}", self.format_value(&constant));
                }
                Ok(OpCode::Pop) => {
                    self.pop();
                }
                Ok(OpCode::DefineGlobal) => self.run_define_global(false)?,
                Ok(OpCode::DefineGlobalLong) => self.run_define_global(true)?,
                Ok(OpCode::GetGlobal) => self.run_get_global(false)?,
                Ok(OpCode::GetGlobalLong) => self.run_get_global(true)?,
                Ok(OpCode::SetGlobal) => self.run_set_global(false)?,
                Ok(OpCode::SetGlobalLong) => self.run_set_global(true)?,
                Ok(OpCode::Return) => self.run_return()?,
                Err(_) => {
                    return Err(InterpretError::Compile(CompileError::InvalidOpCode(
                        self.chunk.get_line(self.ip),
                        op,
                    )))
                }
            }
        }
        Ok(())
    }
}

// bytecode execution functions
impl VM {
    /// Gets the current value in `constant` as specified by the index internal `ip` counter.
    /// If `long` is set to true, retreives the next 3 bytes in `code` to form the index.
    /// Advances the internal `ip` counter pass all the used bytes.
    fn get_constant_operand(&mut self, long: bool) -> Value {
        if long {
            let low_byte = self.chunk.code[self.ip] as usize;
            let mid_byte = self.chunk.code[self.ip + 1] as usize;
            let high_byte = self.chunk.code[self.ip + 2] as usize;
            let constant_idx = (high_byte << 16) | (mid_byte << 8) | low_byte;
            let constant = self.chunk.constants[constant_idx];
            self.ip += 3;
            constant
        } else {
            let constant = self.chunk.constants[self.chunk.code[self.ip] as usize];
            self.ip += 1;
            constant
        }
    }

    fn run_constant(&mut self, long: bool) -> Return {
        self.ip += 1;
        let constant = self.get_constant_operand(long);
        self.push(constant);
        Ok(())
    }

    fn run_negate(&mut self) -> Return {
        let constant = self.pop();
        match constant {
            n if n.is_number() => {
                self.push(Value::number(-n.as_number()));
            }
            _ => {
                return Err(InterpretError::Runtime(RuntimeError::OperandMismatch(
                    self.chunk.get_line(self.ip),
                    "number".to_string(),
                )))
            }
        }

        self.ip += 1;
        Ok(())
    }

    #[inline]
    fn run_not(&mut self) -> Return {
        let constant = self.pop();
        self.push(Value::boolean(!constant.is_truthy()));

        self.ip += 1;
        Ok(())
    }

    fn run_add(&mut self) -> Return {
        let right = self.pop();
        let left = self.pop();
        match (left, right) {
            (n1, n2) if n1.is_number() && n2.is_number() => {
                self.push(Value::number(n1.as_number() + n2.as_number()))
            }
            (s1, s2) if s1.is_object() && s2.is_object() => {
                let s1 = self.get_obj(&s1);
                let s2 = self.get_obj(&s2);

                match (s1, s2) {
                    (Some(Object::String(s1)), Some(Object::String(s2))) => {
                        let s = format!("{s1}{s2}");
                        let value = self.alloc(Object::String(s));
                        self.push(value);
                    }
                    _ => {
                        return Err(InterpretError::Runtime(RuntimeError::OperandMismatch(
                            self.chunk.get_line(self.ip),
                            "numbers or strings".to_string(),
                        )))
                    }
                }
            }
            _ => {
                return Err(InterpretError::Runtime(RuntimeError::OperandMismatch(
                    self.chunk.get_line(self.ip),
                    "numbers or strings".to_string(),
                )))
            }
        }
        Ok(())
    }

    fn run_equals(&mut self) -> Return {
        let right = self.pop();
        let left = self.pop();

        match (left, right) {
            (n1, n2) if n1.is_number() && n2.is_number() => {
                self.push(Value::boolean(n1.as_number() == n2.as_number()))
            }
            (b1, b2) if b1.is_boolean() && b2.is_boolean() => {
                self.push(Value::boolean(b1.as_boolean() == b2.as_boolean()))
            }
            (n1, n2) if n1.is_nil() && n2.is_nil() => self.push(Value::boolean(true)),
            (o1, o2) if o1.is_object() && o2.is_object() => {
                match (self.get_obj(&o1), self.get_obj(&o2)) {
                    (Some(Object::String(s1)), Some(Object::String(s2))) => {
                        self.push(Value::boolean(s1 == s2))
                    }
                    _ => return Err(InterpretError::Deallocated(self.chunk.get_line(self.ip))),
                }
            }
            _ => self.push(Value::boolean(false)),
        }

        self.ip += 1;
        Ok(())
    }

    /// Binary operations that only work on numbers
    fn run_numeric_binary(&mut self, op: OpCode) -> Return {
        let right = self.pop();
        let left = self.pop();
        match (left, right) {
            (n1, n2) if n1.is_number() && n2.is_number() => match op {
                OpCode::Subtract => self.push(Value::number(n1.as_number() - n2.as_number())),
                OpCode::Multiply => self.push(Value::number(n1.as_number() - n2.as_number())),
                OpCode::Divide => self.push(Value::number(n1.as_number() - n2.as_number())),
                OpCode::NotEqual => self.push(Value::boolean(n1.as_number() != n2.as_number())),
                OpCode::LessThan => self.push(Value::boolean(n1.as_number() < n2.as_number())),
                OpCode::LessEqual => self.push(Value::boolean(n1.as_number() <= n2.as_number())),
                OpCode::GreaterThan => self.push(Value::boolean(n1.as_number() > n2.as_number())),
                OpCode::GreaterEqual => self.push(Value::boolean(n1.as_number() >= n2.as_number())),
                _ => {
                    return Err(InterpretError::Panic(
                        self.chunk.get_line(self.ip),
                        format!("Invalid OP_CODE: '{:?}'", op),
                    ))
                }
            },
            _ => {
                return Err(InterpretError::Runtime(RuntimeError::OperandMismatch(
                    self.chunk.get_line(self.ip),
                    "number".to_string(),
                )))
            }
        }

        self.ip += 1;
        Ok(())
    }

    fn run_define_global(&mut self, long: bool) -> Return {
        let ip = self.ip;
        self.ip += 1;
        let name = self.get_constant_operand(long);
        let value = self.pop();

        if name.is_object() {
            match self.get_obj(&name) {
                Some(Object::String(s)) => {
                    self.globals.insert(s.to_string(), value);
                }
                _ => return Err(InterpretError::Deallocated(self.chunk.get_line(ip))),
            }
        } else {
            return Err(InterpretError::Panic(
                self.chunk.get_line(ip),
                "Variable name is not an object (should never run).".to_string(),
            ));
        }

        Ok(())
    }

    fn run_get_global(&mut self, long: bool) -> Return {
        let ip = self.ip;
        self.ip += 1;
        let name = self.get_constant_operand(long);

        if name.is_object() {
            match self.get_obj(&name) {
                Some(Object::String(s)) => {
                    let value = self.globals.get(s);
                    match value {
                        Some(v) => {
                            self.push(*v);
                        }
                        None => {
                            return Err(InterpretError::Runtime(RuntimeError::NameError(
                                self.chunk.get_line(ip),
                                s.to_string(),
                            )))
                        }
                    }
                }
                _ => return Err(InterpretError::Deallocated(self.chunk.get_line(ip))),
            }
        } else {
            return Err(InterpretError::Panic(
                self.chunk.get_line(ip),
                "Variable name is not an object (should never run).".to_string(),
            ));
        }

        Ok(())
    }

    fn run_set_global(&mut self, long: bool) -> Return {
        let ip = self.ip;
        self.ip += 1;
        let name = self.get_constant_operand(long);
        let value = *self.stack.last().unwrap_or(&Value::nil());

        if name.is_object() {
            match self.get_obj(&name) {
                Some(Object::String(s)) => {
                    if self.globals.contains_key(s) {
                        self.globals.insert(s.to_string(), value);
                    } else {
                        return Err(InterpretError::Runtime(RuntimeError::NameError(
                            self.chunk.get_line(ip),
                            s.to_string(),
                        )));
                    }
                }
                _ => return Err(InterpretError::Deallocated(self.chunk.get_line(ip))),
            }
        } else {
            return Err(InterpretError::Panic(
                self.chunk.get_line(ip),
                "Variable name is not an object (should never run).".to_string(),
            ));
        }
        Ok(())
    }

    fn run_return(&mut self) -> Return {
        Ok(())
    }
}
