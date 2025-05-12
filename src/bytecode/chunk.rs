use crate::core::{OpCode, Value};

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

    pub fn disassemble(&self, name: &str) {
        eprintln!("== {} ==", name);
        let mut offset = 0;

        let len = self.code.len();
        while offset < len {
            offset = self.disassemble_instruction(offset);
        }
    }

    pub fn disassemble_instruction(&self, mut offset: usize) -> usize {
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
                | OpCode::SetGlobal => self.disassemble_constant_instruction(op, offset),
                OpCode::LoadConstantLong
                | OpCode::DefineGlobalLong
                | OpCode::GetGlobalLong
                | OpCode::SetGlobalLong => self.disassemble_constant_long_instruction(op, offset),
                OpCode::GetLocal | OpCode::SetLocal | OpCode::Call | OpCode::Closure => {
                    self.disassemble_num_instruction(op, offset)
                }
                OpCode::GetLocalLong | OpCode::SetLocalLong | OpCode::ClosureLong => {
                    self.disassemble_num_long_instruction(op, offset)
                }
                OpCode::Jump | OpCode::JumpIfFalse | OpCode::Loop => {
                    self.disassemble_num_mid_instruction(op, offset)
                }
                _ => self.disassemble_simple_instruction(op),
            },
            Err(_) => {
                eprintln!("Invalid Opcode '{}'", instruction);
                1
            }
        };

        offset
    }

    fn read_operand(&self, operands: u8, offset: usize) -> usize {
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

    fn disassemble_constant_instruction(&self, op: OpCode, offset: usize) -> usize {
        let constant_idx = self.read_operand(1, offset);
        let constant = self.constants[constant_idx];
        eprintln!("{:<16?} {:>4} '{:?}'", op, constant_idx, constant);
        2
    }

    fn disassemble_constant_long_instruction(&self, op: OpCode, offset: usize) -> usize {
        let constant_idx = self.read_operand(3, offset);
        let constant = self.constants.get(constant_idx).unwrap();
        eprintln!("{:<16?} {:>4} '{:?}'", op, constant_idx, constant);
        4
    }

    fn disassemble_num_instruction(&self, op: OpCode, offset: usize) -> usize {
        let constant_idx = self.read_operand(1, offset);
        eprintln!("{:<16?} {:>4}", op, constant_idx);
        2
    }

    fn disassemble_num_mid_instruction(&self, op: OpCode, offset: usize) -> usize {
        let constant_idx = self.read_operand(2, offset);
        eprintln!("{:<16?} {:>4}", op, constant_idx);
        3
    }

    fn disassemble_num_long_instruction(&self, op: OpCode, offset: usize) -> usize {
        let constant_idx = self.read_operand(3, offset);
        eprintln!("{:<16?} {:>4}", op, constant_idx);
        4
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}
