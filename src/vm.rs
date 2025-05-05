use crate::{core::value::Object, opcode::OpCode, Chunk, InterpretError, Value};

pub struct VM {
    ip: usize,
    stack: Vec<Value>,
    heap: Vec<Object>,
}

impl VM {
    const STACK_MAX: usize = 256;
    pub fn new() -> Self {
        VM {
            ip: 0,
            stack: Vec::with_capacity(Self::STACK_MAX),
            heap: vec![],
        }
    }

    pub fn run(&mut self, chunk: &Chunk) -> Result<(), InterpretError> {
        let code = chunk.code();
        let constants = chunk.constants();

        while self.ip < code.len() {
            let op = code[self.ip];

            #[cfg(debug_assertions)]
            {
                self.stack_dump();
                chunk.disassemble_instruction(self.ip);
            }

            match OpCode::try_from(op) {
                Ok(OpCode::Constant) => {
                    let constant = constants[code[self.ip + 1] as usize];
                    self.push(constant);
                    self.ip += 2;
                }
                Ok(OpCode::ConstantLong) => {
                    let low_byte = code[self.ip + 1] as usize;
                    let mid_byte = code[self.ip + 2] as usize;
                    let high_byte = code[self.ip + 3] as usize;
                    let constant_idx = (high_byte << 16) | (mid_byte << 8) | low_byte;
                    let constant = constants[constant_idx];
                    self.push(constant);
                    self.ip += 3;
                }
                Ok(OpCode::Negate) => {
                    let constant = self.pop();
                    match constant {
                        n if n.is_number() => {
                            self.push(Value::number(-n.as_number()));
                        }
                        _ => todo!(),
                    }
                    self.ip += 1;
                }
                Ok(OpCode::Add) => {
                    let err = Err(InterpretError::RuntimeError);

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
                                _ => return err,
                            }
                        }
                        _ => return err,
                    }
                    self.ip += 1;
                }
                Ok(OpCode::Subtract) => {
                    let right = self.pop();
                    let left = self.pop();
                    match (left, right) {
                        (n1, n2) if n1.is_number() && n2.is_number() => {
                            self.push(Value::number(n1.as_number() - n2.as_number()))
                        }
                        _ => todo!(),
                    }
                    self.ip += 1;
                }
                Ok(OpCode::Multiply) => {
                    let right = self.pop();
                    let left = self.pop();
                    match (left, right) {
                        (n1, n2) if n1.is_number() && n2.is_number() => {
                            self.push(Value::number(n1.as_number() * n2.as_number()))
                        }
                        _ => todo!(),
                    }
                    self.ip += 1;
                }
                Ok(OpCode::Divide) => {
                    let right = self.pop();
                    let left = self.pop();
                    match (left, right) {
                        (n1, n2) if n1.is_number() && n2.is_number() => {
                            self.push(Value::number(n1.as_number() / n2.as_number()))
                        }
                        _ => todo!(),
                    }
                    self.ip += 1;
                }
                Ok(OpCode::Return) => {
                    let constant = self.pop();
                    println!("{}", self.format_value(&constant));
                    self.ip += 1;
                    return Ok(());
                }
                Err(_) => return Err(InterpretError::CompileError),
            }
        }
        Ok(())
    }

    /// Pushes a new value at the back of the stack
    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    /// Removes and returns the last value on the stack
    pub fn pop(&mut self) -> Value {
        self.stack.pop().unwrap_or(Value::nil())
    }

    pub fn alloc(&mut self, obj: Object) -> Value {
        self.heap.push(obj);

        Value::object(self.heap.len() - 1)
    }

    pub fn get_obj(&self, value: &Value) -> Option<&Object> {
        if !value.is_object() {
            return None;
        }

        self.heap.get(value.as_object())
    }

    pub fn stack_dump(&self) {
        print!("          ");
        for value in &self.stack {
            print!("[ {} ]", self.format_value(&value))
        }
        println!("");
    }

    pub fn format_value(&self, value: &Value) -> String {
        if value.is_object() {
            match self.get_obj(value) {
                Some(Object::String(s)) => format!("{}", s),
                None => format!("nil"),
            }
        } else if value.is_number() {
            format!("{}", value.as_number())
        } else if value.is_boolean() {
            format!("{}", value.as_boolean())
        } else if value.is_nil() {
            format!("nil")
        } else {
            format!("Invalid value format!")
        }
    }
}
