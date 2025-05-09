use crate::{core::value::Value, opcode::OpCode};

#[derive(Clone)]
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
        println!("== {} ==", name);
        let mut offset = 0;

        let len = self.code.len();
        while offset < len {
            offset = self.disassemble_instruction(offset);
        }
    }

    pub fn disassemble_instruction(&self, mut offset: usize) -> usize {
        let instruction = self.code[offset];
        let line = self.get_line(offset);

        print!(
            "{:04} {}",
            offset,
            if offset > 0 && line == self.get_line(offset - 1) {
                "   | ".to_string()
            } else {
                format!("{:>4} ", line)
            }
        );

        offset += 1;
        match OpCode::try_from(instruction) {
            Ok(op) => match op {
                OpCode::LoadConstant
                | OpCode::DefineGlobal
                | OpCode::GetGlobal
                | OpCode::SetGlobal => offset = self.disassemble_constant_instruction(op, offset),
                OpCode::LoadConstantLong
                | OpCode::DefineGlobalLong
                | OpCode::GetGlobalLong
                | OpCode::SetGlobalLong => offset = self.disassemble_long_instruction(op, offset),
                _ => self.disassemble_simple_instruction(op),
            },
            Err(_) => {
                println!("Invalid Opcode '{}'", instruction);
            }
        };

        offset
    }

    fn disassemble_simple_instruction(&self, op: OpCode) {
        println!("{:?}", op);
    }

    fn disassemble_constant_instruction(&self, op: OpCode, mut offset: usize) -> usize {
        let constant_idx = self.code[offset] as usize;
        let constant = self.constants[constant_idx];
        println!("{:<16?} {:>4} '{:?}'", op, constant_idx, constant);
        offset += 1;
        offset
    }

    fn disassemble_long_instruction(&self, op: OpCode, mut offset: usize) -> usize {
        let low_byte = self.code[offset] as usize;
        let mid_byte = self.code[offset + 1] as usize;
        let high_byte = self.code[offset + 1] as usize;
        let constant_idx = (high_byte << 16) | (mid_byte << 8) | low_byte;
        let constant = self.constants.get(constant_idx).unwrap();
        println!("{:<16?} {:>4} '{:?}'", op, constant_idx, constant);
        offset += 3;
        offset
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}
