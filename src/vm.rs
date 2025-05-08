use std::collections::HashMap;

use crate::{
    core::{
        errors::{CompileError, InterpretError, PanicError, RuntimeError},
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
                    self.ip += 1;
                }
                Ok(OpCode::Pop) => {
                    self.pop();
                    self.ip += 1;
                }
                Ok(OpCode::DefineGlobal) => self.run_define_global(false)?,
                Ok(OpCode::DefineGlobalLong) => self.run_define_global(true)?,
                Ok(OpCode::GetGlobal) => self.run_get_global(false)?,
                Ok(OpCode::GetGlobalLong) => self.run_get_global(true)?,
                Ok(OpCode::SetGlobal) => self.run_set_global(false)?,
                Ok(OpCode::SetGlobalLong) => self.run_set_global(true)?,
                Ok(OpCode::GetLocal) => self.run_get_local(false)?,
                Ok(OpCode::GetLocalLong) => self.run_get_local(true)?,
                Ok(OpCode::SetLocal) => self.run_set_local(false)?,
                Ok(OpCode::SetLocalLong) => self.run_set_local(true)?,
                Ok(OpCode::Return) => self.run_return()?,
                Err(_) => {
                    self.ip += 1;
                    return Err(InterpretError::Compile(CompileError::InvalidOpCode(
                        self.chunk.get_line(self.ip),
                        op,
                    )));
                }
            }
        }
        Ok(())
    }
}

// bytecode execution functions
impl VM {
    /// Reads the operand at the current position of the internal `ip` counter.
    /// If `long` is set to true, retrieves the next 3 bytes to form the operand, otherwise
    /// only consumes the current byte. Advances the interal `ip` counter pass all the
    /// bytes read.
    fn read_operand(&mut self, long: bool) -> usize {
        if long {
            let low_byte = self.chunk.code[self.ip] as usize;
            let mid_byte = self.chunk.code[self.ip + 1] as usize;
            let high_byte = self.chunk.code[self.ip + 2] as usize;
            self.ip += 3;
            (high_byte << 16) | (mid_byte << 8) | low_byte
        } else {
            self.ip += 1;
            self.chunk.code[self.ip - 1] as usize
        }
    }

    fn run_constant(&mut self, long: bool) -> Return {
        self.ip += 1;
        let index = self.read_operand(long);
        self.push(self.chunk.constants[index]);
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

        self.ip += 1;
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
                    _ => {
                        return Err(InterpretError::Panic(PanicError::DeallocatedObject(
                            self.chunk.get_line(self.ip),
                        )))
                    }
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
                    return Err(InterpretError::Panic(PanicError::General(
                        self.chunk.get_line(self.ip),
                        format!("Invalid OP_CODE: '{:?}'", op),
                    )))
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

    fn get_variable_name(&mut self, name: &Value, ip: usize) -> Result<String, InterpretError> {
        if name.is_object() {
            match self.get_obj(&name) {
                Some(Object::String(s)) => return Ok(s.to_string()),
                _ => {
                    return Err(InterpretError::Panic(PanicError::DeallocatedObject(
                        self.chunk.get_line(ip),
                    )))
                }
            }
        } else {
            return Err(InterpretError::Panic(PanicError::NonObjectVariable(
                self.chunk.get_line(ip),
            )));
        }
    }

    fn run_define_global(&mut self, long: bool) -> Return {
        let value = self.pop();

        let ip = self.ip;
        self.ip += 1;
        let index = self.read_operand(long);

        let name_value = self.chunk.constants[index];
        let name = self.get_variable_name(&name_value, ip)?;

        self.globals.insert(name, value);

        Ok(())
    }

    fn run_get_global(&mut self, long: bool) -> Return {
        let ip = self.ip;
        self.ip += 1;
        let index = self.read_operand(long);

        let name_value = self.chunk.constants[index];
        let name = self.get_variable_name(&name_value, ip)?;

        let value = self.globals.get(&name);
        match value {
            Some(v) => {
                self.push(*v);
            }
            None => {
                return Err(InterpretError::Runtime(RuntimeError::NameError(
                    self.chunk.get_line(ip),
                    name,
                )))
            }
        }

        Ok(())
    }

    fn run_set_global(&mut self, long: bool) -> Return {
        let value = *self.stack.last().unwrap_or(&Value::nil());

        let ip = self.ip;
        self.ip += 1;
        let index = self.read_operand(long);

        let name_value = self.chunk.constants[index];
        let name = self.get_variable_name(&name_value, ip)?;

        if self.globals.contains_key(&name) {
            self.globals.insert(name, value);
        } else {
            return Err(InterpretError::Runtime(RuntimeError::NameError(
                self.chunk.get_line(ip),
                name,
            )));
        }

        Ok(())
    }

    fn run_get_local(&mut self, long: bool) -> Return {
        self.ip += 1;
        let index = self.read_operand(long);
        self.push(self.stack[index]);
        Ok(())
    }

    fn run_set_local(&mut self, long: bool) -> Return {
        self.ip += 1;
        let index = self.read_operand(long);
        self.stack[index] = *self.stack.last().unwrap_or(&Value::nil());

        Ok(())
    }

    fn run_return(&mut self) -> Return {
        self.ip += 1;
        Ok(())
    }
}
