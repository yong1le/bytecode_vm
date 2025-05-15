use std::sync::atomic::compiler_fence;

use crate::{
    core::{OpCode, Value},
    object::Object,
    VM,
};

pub struct Chunk {
    pub code: Vec<u8>,
    /// Run-length encoding of line numbers
    /// <https://en.wikipedia.org/wiki/Run-length_encoding>
    pub lines: Vec<(u32, usize)>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    // Writes a single byte to the code instructions array
    pub fn write_byte(&mut self, byte: u8, line: u32) {
        self.code.push(byte);

        if let Some(last_line) = self.lines.last_mut() {
            if last_line.0 == line {
                last_line.1 += 1;
            } else {
                self.lines.push((line, 1));
            }
        } else {
            self.lines.push((line, 1))
        }
    }

    // Adds a constant to the chunk's constant pool.
    //
    // Returns the index of the constant in the constant pool.
    pub fn add_constant(&mut self, constant: Value) -> usize {
        self.constants.push(constant);
        self.constants.len() - 1
    }

    pub fn get_line(&self, mut offset: usize) -> u32 {
        for line in &self.lines {
            if offset >= line.1 {
                offset -= line.1;
            } else {
                return line.0;
            }
        }
        0
    }

    pub fn disassemble(&self, name: &str, vm: &VM) {
        eprintln!("== {} ==", name);
        let mut offset = 0;

        let len = self.code.len();
        while offset < len {
            offset = self.disassemble_instruction(offset, vm);
        }
    }

    pub fn disassemble_instruction(&self, mut offset: usize, vm: &VM) -> usize {
        let instruction = self.code[offset];
        let line = self.get_line(offset);

        eprint!(
            "{:04} {}",
            offset,
            if offset > 0 && line == self.get_line(offset - 1) {
                "   | ".to_string()
            } else {
                format!("{:>4} ", line)
            }
        );

        offset += match OpCode::try_from(instruction) {
            Ok(op) => match op {
                OpCode::LoadConstant
                | OpCode::DefineGlobal
                | OpCode::GetGlobal
                | OpCode::SetGlobal => self.disassemble_constant_instruction(op, 1, offset, vm),
                OpCode::LoadConstantLong
                | OpCode::DefineGlobalLong
                | OpCode::GetGlobalLong
                | OpCode::SetGlobalLong => self.disassemble_constant_instruction(op, 3, offset, vm),
                OpCode::GetLocal | OpCode::SetLocal => {
                    self.disassemble_stack_instruction(op, 1, offset, vm)
                }
                OpCode::GetLocalLong | OpCode::SetLocalLong => {
                    self.disassemble_stack_instruction(op, 3, offset, vm)
                }
                OpCode::Call => self.disassemble_num_instruction(op, 1, offset),
                OpCode::Jump | OpCode::JumpIfFalse | OpCode::Loop => {
                    self.disassemble_num_instruction(op, 2, offset)
                }
                OpCode::GetUpvalue | OpCode::SetUpvalue => {
                    self.disassemble_upvalue_instruction(op, 1, offset, vm)
                }
                OpCode::Closure => self.disassemble_closure(op, 1, offset, vm),
                _ => self.disassemble_simple_instruction(op),
            },
            Err(_) => {
                eprintln!("Invalid Opcode '{}'", instruction);
                1
            }
        };

        offset
    }

    fn read_operand(&self, operands: usize, offset: usize) -> usize {
        if operands == 3 {
            let low_byte = self.code[offset + 1] as usize;
            let mid_byte = self.code[offset + 2] as usize;
            let high_byte = self.code[offset + 3] as usize;
            (high_byte << 16) | (mid_byte << 8) | low_byte
        } else if operands == 2 {
            let low_byte = self.code[offset + 1] as usize;
            let high_byte = self.code[offset + 2] as usize;
            (high_byte << 8) | low_byte
        } else if operands == 1 {
            self.code[offset + 1] as usize
        } else {
            panic!("<read_operand> only acepts 1, 2, or 3")
        }
    }

    fn disassemble_simple_instruction(&self, op: OpCode) -> usize {
        eprintln!("{:?}", op);
        1
    }

    /// Disassemble instruction that indexes into the constant pool
    fn disassemble_constant_instruction(
        &self,
        op: OpCode,
        operands: usize,
        offset: usize,
        vm: &VM,
    ) -> usize {
        let constant_idx = self.read_operand(operands, offset);
        let constant = self.constants[constant_idx];
        eprintln!(
            "{:<16?} {:>4} '{:?}'",
            op,
            constant_idx,
            vm.format_value(&constant)
        );
        operands + 1
    }

    /// Disasemble instruction that indexes into the VM stack
    fn disassemble_stack_instruction(
        &self,
        op: OpCode,
        operands: usize,
        offset: usize,
        vm: &VM,
    ) -> usize {
        let stack_idx = self.read_operand(operands, offset);
        let stack_value = vm.stack_get(stack_idx);
        eprintln!(
            "{:<16?} {:>4} '{:}'",
            op,
            stack_idx,
            vm.format_value(&stack_value)
        );
        operands + 1
    }

    /// Disassemble instruction that indexes into the current frame's upvalues array
    fn disassemble_upvalue_instruction(
        &self,
        op: OpCode,
        operands: usize,
        offset: usize,
        vm: &VM,
    ) -> usize {
        let upvalue_idx = self.read_operand(operands, offset);
        let upvalue = vm.upvalue_get(upvalue_idx as u8);
        eprintln!(
            "{:<16?} {:>4} '{}'",
            op,
            upvalue_idx,
            vm.format_value(&upvalue)
        );
        operands + 1
    }

    // Disassemble instruction that takes a number as an argument (rather than indexing somehwere).
    fn disassemble_num_instruction(&self, op: OpCode, operands: usize, offset: usize) -> usize {
        let number = self.read_operand(operands, offset);
        eprintln!("{:<16?} {:>4}", op, number);
        operands + 1
    }

    fn disassemble_closure(&self, op: OpCode, operands: usize, offset: usize, vm: &VM) -> usize {
        let mut operands = operands;
        let heap_idx = self.read_operand(operands, offset);
        operands += 1;

        let function_idx = Value::object(heap_idx);
        eprintln!(
            "{:<16?} {:>4} '{}'",
            op,
            heap_idx,
            vm.format_value(&function_idx)
        );
        if let Some(Object::Function(function)) = vm.heap_get(&function_idx) {
            for _ in 0..function.upvalue_count {
                operands += 2;
            }
        } else {
            panic!("Closure on non function.")
        }

        operands
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}
