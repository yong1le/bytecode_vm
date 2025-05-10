use std::{collections::HashMap, io::Write};

use super::{frame::Frame, heap::Heap, Return, FRAME_MAX, STACK_MAX, VM};
use crate::{
    bytecode::Chunk,
    core::{
        errors::{CompileError, InterpretError, PanicError, RuntimeError},
        OpCode, Value,
    },
    object::Object,
};

impl<'a> VM<'a> {
    pub fn new(writer: Box<dyn Write + 'a>) -> Self {
        Self {
            frames: Vec::with_capacity(FRAME_MAX),
            stack: Vec::with_capacity(STACK_MAX),
            heap: Heap::new(),
            globals: HashMap::new(),
            writer,
        }
    }

    fn get_ip(&self) -> usize {
        self.get_frame().ip
    }

    fn increment_ip(&mut self, offset: usize) {
        self.get_frame_mut().ip += offset;
    }

    fn decrement_ip(&mut self, offset: usize) {
        self.get_frame_mut().ip -= offset;
    }

    fn get_chunk(&self) -> &Chunk {
        &self.get_frame().function.chunk
    }

    fn get_code_length(&self) -> usize {
        self.get_frame().function.chunk.code.len()
    }

    fn get_current_line(&self) -> u32 {
        let ip = self.get_ip();
        self.get_chunk().get_line(ip)
    }

    pub(crate) fn format_value(&self, value: &Value) -> String {
        if value.is_object() {
            match self.heap_get(value) {
                Some(Object::String(s)) => s.to_string(),
                Some(Object::Function(f)) => format!("<fn {}>", f.name),
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
}

// bytecode execution functions
impl VM<'_> {
    pub fn run(&mut self, frame: Frame) -> Return {
        self.frames.push(frame);
        self.stack.push(Value::number(0.0));

        while self.get_ip() < self.get_code_length() {
            let ip = self.get_ip();
            let op = self.get_chunk().code[ip];

            #[cfg(debug_assertions)]
            {
                self.stack_dump();
                self.heap.dump();
                self.get_chunk().disassemble_instruction(ip);
            }

            match OpCode::try_from(op) {
                Ok(OpCode::LoadConstant) => self.run_constant(1)?,
                Ok(OpCode::LoadConstantLong) => self.run_constant(3)?,
                Ok(OpCode::Negate) => self.run_negate()?,
                Ok(OpCode::Not) => self.run_not()?,
                Ok(OpCode::Add) => self.run_add()?,
                Ok(OpCode::Subtract) => self.run_numeric_binary(OpCode::Subtract)?,
                Ok(OpCode::Multiply) => self.run_numeric_binary(OpCode::Multiply)?,
                Ok(OpCode::Divide) => self.run_numeric_binary(OpCode::Divide)?,
                Ok(OpCode::Equal) => self.run_equals(true)?,
                Ok(OpCode::NotEqual) => self.run_equals(false)?,
                Ok(OpCode::LessEqual) => self.run_numeric_binary(OpCode::LessEqual)?,
                Ok(OpCode::LessThan) => self.run_numeric_binary(OpCode::LessThan)?,
                Ok(OpCode::GreaterThan) => self.run_numeric_binary(OpCode::GreaterThan)?,
                Ok(OpCode::GreaterEqual) => self.run_numeric_binary(OpCode::GreaterEqual)?,
                Ok(OpCode::Print) => {
                    let constant = self.stack_pop();
                    writeln!(self.writer, "{}", self.format_value(&constant)).unwrap();
                    self.increment_ip(1);
                }
                Ok(OpCode::Pop) => {
                    self.stack_pop();
                    self.increment_ip(1);
                }
                Ok(OpCode::DefineGlobal) => self.run_define_global(1)?,
                Ok(OpCode::DefineGlobalLong) => self.run_define_global(3)?,
                Ok(OpCode::GetGlobal) => self.run_get_global(1)?,
                Ok(OpCode::GetGlobalLong) => self.run_get_global(3)?,
                Ok(OpCode::SetGlobal) => self.run_set_global(1)?,
                Ok(OpCode::SetGlobalLong) => self.run_set_global(3)?,
                Ok(OpCode::GetLocal) => self.run_get_local(1)?,
                Ok(OpCode::GetLocalLong) => self.run_get_local(3)?,
                Ok(OpCode::SetLocal) => self.run_set_local(1)?,
                Ok(OpCode::SetLocalLong) => self.run_set_local(3)?,
                Ok(OpCode::JumpIfFalse) => {
                    self.increment_ip(1);
                    let jump_distance = self.read_operand(2);
                    let condition = self.stack_peek(0);

                    if !condition.is_truthy() {
                        self.increment_ip(jump_distance);
                    }
                }
                Ok(OpCode::Jump) => {
                    self.increment_ip(1);
                    let jump_distance = self.read_operand(2);
                    self.increment_ip(jump_distance);
                }
                Ok(OpCode::Loop) => {
                    self.increment_ip(1);
                    let jump_distance = self.read_operand(2);
                    self.decrement_ip(jump_distance);
                }
                Ok(OpCode::Call) => {
                    self.increment_ip(1);
                    let argc = self.read_operand(1);
                    let callee = self.stack_peek(argc);
                    if callee.is_object() {
                        match self.heap_get(&callee) {
                            Some(Object::Function(f)) => {
                                // FIXME: cloning a function is too expensive, cosnider using an arena
                                self.frames
                                    .push(Frame::new(f.clone(), self.stack.len() - argc - 1));
                            }
                            Some(_) => {
                                return Err(InterpretError::Runtime(RuntimeError::InvalidCall(
                                    self.get_current_line(),
                                    self.format_value(&callee),
                                )));
                            }
                            None => {
                                return Err(InterpretError::Panic(PanicError::DeallocatedObject(
                                    self.get_current_line(),
                                )))
                            }
                        }
                    } else {
                        return Err(InterpretError::Runtime(RuntimeError::InvalidCall(
                            self.get_current_line(),
                            self.format_value(&callee),
                        )));
                    }
                }
                Ok(OpCode::Return) => {
                    self.increment_ip(1);
                    let return_val = self.stack_pop();

                    let popped_frame = self.frames.pop().unwrap();
                    if self.frames.is_empty() {
                        self.stack_pop();
                        return Ok(());
                    }

                    self.stack.truncate(popped_frame.fp);
                    self.stack_push(return_val);
                }
                Ok(OpCode::Nop) => self.increment_ip(1),
                Err(_) => {
                    self.increment_ip(1);
                    return Err(InterpretError::Compile(CompileError::InvalidOpCode(
                        self.get_current_line(),
                        op,
                    )));
                }
            }
        }
        Ok(())
    }

    /// Reads the operand at the current position of the internal `ip` counter.
    /// If `long` is set to true, retrieves the next 3 bytes to form the operand, otherwise
    /// only consumes the current byte. Advances the interal `ip` counter pass all the
    /// bytes read.
    fn read_operand(&mut self, operands: u8) -> usize {
        let ip = self.get_ip();
        let code = &self.get_chunk().code;

        if operands == 3 {
            let low_byte = code[ip] as usize;
            let mid_byte = code[ip + 1] as usize;
            let high_byte = code[ip + 2] as usize;
            self.increment_ip(3);
            (high_byte << 16) | (mid_byte << 8) | low_byte
        } else if operands == 2 {
            let low_byte = code[ip] as usize;
            let high_byte = code[ip + 1] as usize;
            self.increment_ip(2);
            (high_byte << 8) | low_byte
        } else if operands == 1 {
            let byte = code[ip] as usize;
            self.increment_ip(1);
            byte
        } else {
            panic!("<read_operand> only acepts 1, 2, or 3")
        }
    }

    fn run_constant(&mut self, operands: u8) -> Return {
        self.increment_ip(1);
        let index = self.read_operand(operands);
        let constant = self.get_chunk().constants[index];
        self.stack_push(constant);
        Ok(())
    }

    fn run_negate(&mut self) -> Return {
        let constant = self.stack_pop();
        match constant {
            n if n.is_number() => {
                self.stack_push(Value::number(-n.as_number()));
            }
            _ => {
                return Err(InterpretError::Runtime(RuntimeError::OperandMismatch(
                    self.get_current_line(),
                    "numbers".to_string(),
                )));
            }
        }

        self.increment_ip(1);
        Ok(())
    }

    #[inline]
    fn run_not(&mut self) -> Return {
        let constant = self.stack_pop();
        self.stack_push(Value::boolean(!constant.is_truthy()));

        self.increment_ip(1);
        Ok(())
    }

    fn run_add(&mut self) -> Return {
        let right = self.stack_pop();
        let left = self.stack_pop();
        match (left, right) {
            (n1, n2) if n1.is_number() && n2.is_number() => {
                self.stack_push(Value::number(n1.as_number() + n2.as_number()))
            }
            (s1, s2) if s1.is_object() && s2.is_object() => {
                let s1 = self.heap_get(&s1);
                let s2 = self.heap_get(&s2);

                match (s1, s2) {
                    (Some(Object::String(s1)), Some(Object::String(s2))) => {
                        let s = format!("{s1}{s2}");
                        let value = self.heap_alloc(Object::String(s));
                        self.stack_push(value);
                    }
                    _ => {
                        return Err(InterpretError::Runtime(RuntimeError::OperandMismatch(
                            self.get_current_line(),
                            "numbers or strings".to_string(),
                        )));
                    }
                }
            }
            _ => {
                return Err(InterpretError::Runtime(RuntimeError::OperandMismatch(
                    self.get_current_line(),
                    "numbers or strings".to_string(),
                )));
            }
        }

        self.increment_ip(1);
        Ok(())
    }

    fn run_equals(&mut self, equality: bool) -> Return {
        let right = self.stack_pop();
        let left = self.stack_pop();

        match (left, right) {
            (n1, n2) if n1.is_number() && n2.is_number() => {
                self.stack_push(Value::boolean(if equality {
                    n1.as_number() == n2.as_number()
                } else {
                    n1.as_number() != n2.as_number()
                }))
            }
            (b1, b2) if b1.is_boolean() && b2.is_boolean() => {
                self.stack_push(Value::boolean(if equality {
                    b1.as_boolean() == b2.as_boolean()
                } else {
                    b1.as_boolean() != b2.as_boolean()
                }))
            }
            (n1, n2) if n1.is_nil() && n2.is_nil() => self.stack_push(Value::boolean(equality)),
            (o1, o2) if o1.is_object() && o2.is_object() => {
                match (self.heap_get(&o1), self.heap_get(&o2)) {
                    (Some(Object::String(s1)), Some(Object::String(s2))) => {
                        self.stack_push(Value::boolean(if equality { s1 == s2 } else { s1 != s2 }))
                    }
                    _ => {
                        return Err(InterpretError::Panic(PanicError::DeallocatedObject(
                            self.get_current_line(),
                        )));
                    }
                }
            }
            _ => self.stack_push(Value::boolean(!equality)),
        }

        self.increment_ip(1);
        Ok(())
    }

    /// Binary operations that only work on numbers
    fn run_numeric_binary(&mut self, op: OpCode) -> Return {
        let right = self.stack_pop();
        let left = self.stack_pop();
        match (left, right) {
            (n1, n2) if n1.is_number() && n2.is_number() => match op {
                OpCode::Subtract => self.stack_push(Value::number(n1.as_number() - n2.as_number())),
                OpCode::Multiply => self.stack_push(Value::number(n1.as_number() * n2.as_number())),
                OpCode::Divide => self.stack_push(Value::number(n1.as_number() / n2.as_number())),
                OpCode::LessThan => {
                    self.stack_push(Value::boolean(n1.as_number() < n2.as_number()))
                }
                OpCode::LessEqual => {
                    self.stack_push(Value::boolean(n1.as_number() <= n2.as_number()))
                }
                OpCode::GreaterThan => {
                    self.stack_push(Value::boolean(n1.as_number() > n2.as_number()))
                }
                OpCode::GreaterEqual => {
                    self.stack_push(Value::boolean(n1.as_number() >= n2.as_number()))
                }
                _ => {
                    return Err(InterpretError::Panic(PanicError::General(
                        self.get_current_line(),
                        format!("Invalid OP_CODE: '{:?}'", op),
                    )))
                }
            },
            _ => {
                return Err(InterpretError::Runtime(RuntimeError::OperandMismatch(
                    self.get_current_line(),
                    "numbers".to_string(),
                )))
            }
        }

        self.increment_ip(1);
        Ok(())
    }

    fn get_variable_name(&mut self, name: &Value, ip: usize) -> Result<String, InterpretError> {
        if name.is_object() {
            match self.heap_get(name) {
                Some(Object::String(s)) => Ok(s.to_string()),
                _ => Err(InterpretError::Panic(PanicError::DeallocatedObject(
                    self.get_chunk().get_line(ip),
                ))),
            }
        } else {
            Err(InterpretError::Panic(PanicError::NonObjectVariable(
                self.get_chunk().get_line(ip),
            )))
        }
    }

    fn run_define_global(&mut self, operands: u8) -> Return {
        let value = self.stack_pop();

        let ip = self.get_ip();
        self.increment_ip(1);
        let index = self.read_operand(operands);

        let name_value = self.get_chunk().constants[index];
        let name = self.get_variable_name(&name_value, ip)?;

        self.globals.insert(name, value);

        Ok(())
    }

    fn run_get_global(&mut self, operands: u8) -> Return {
        let ip = self.get_ip();
        self.increment_ip(1);
        let index = self.read_operand(operands);

        let name_value = self.get_chunk().constants[index];
        let name = self.get_variable_name(&name_value, ip)?;

        let value = self.globals.get(&name);
        match value {
            Some(v) => {
                self.stack_push(*v);
            }
            None => {
                return Err(InterpretError::Runtime(RuntimeError::NameError(
                    self.get_current_line(),
                    name,
                )))
            }
        }

        Ok(())
    }

    fn run_set_global(&mut self, operands: u8) -> Return {
        let value = self.stack_peek(0);

        let ip = self.get_ip();
        self.increment_ip(1);
        let index = self.read_operand(operands);

        let name_value = self.get_chunk().constants[index];
        let name = self.get_variable_name(&name_value, ip)?;

        match self.globals.contains_key(&name) {
            true => {
                self.globals.insert(name, value);
            }
            false => {
                return Err(InterpretError::Runtime(RuntimeError::NameError(
                    self.get_current_line(),
                    name,
                )));
            }
        }

        Ok(())
    }

    fn run_get_local(&mut self, operands: u8) -> Return {
        self.increment_ip(1);
        let index = self.read_operand(operands);
        self.stack_push(self.stack_get(index));
        Ok(())
    }

    fn run_set_local(&mut self, operands: u8) -> Return {
        self.increment_ip(1);
        let index = self.read_operand(operands);
        self.stack_set(index, self.stack_peek(0));

        Ok(())
    }
}
