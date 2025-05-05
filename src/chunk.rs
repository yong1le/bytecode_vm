use crate::{core::value::Value, OpCode};

#[derive(Clone)]
pub struct Chunk {
    code: Vec<u8>,
    /// Run-length encoding of line numbers
    /// <https://en.wikipedia.org/wiki/Run-length_encoding>
    lines: Vec<(u32, usize)>,
    constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }
    pub fn constants(&self) -> &[Value] {
        &self.constants
    }

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

    pub fn write_constant(&mut self, value: Value, line: u32) {
        let constant_idx = self.add_constant(value);

        if constant_idx > 255 {
            self.write_byte(OpCode::ConstantLong as u8, line);
            self.write_byte((constant_idx & 255) as u8, line);
            self.write_byte(((constant_idx >> 8) & 255) as u8, line);
            self.write_byte(((constant_idx >> 16) & 255) as u8, line);
        } else {
            self.write_byte(OpCode::Constant as u8, line);
            self.write_byte(constant_idx as u8, line);
        }
    }

    // Adds a constant to the chunk's constant pool.
    //
    // Returns the index of the constant in the constant pool.
    fn add_constant(&mut self, constant: Value) -> usize {
        self.constants.push(constant);
        self.constants.len() - 1
    }

    pub fn get_line(&self, mut offset: usize) -> u32 {
        for line in &self.lines {
            if offset >= line.1 {
                offset = offset - line.1;
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
        match OpCode::try_from(instruction) {
            Ok(OpCode::Constant) => {
                let constant_idx = self.code[offset + 1] as usize;
                let constant = self.constants.get(constant_idx).unwrap();
                println!("{:<16} {:>4} '{:?}'", "OP_CONSTANT", constant_idx, constant);
                offset += 2;
            }
            Ok(OpCode::ConstantLong) => {
                let low_byte = self.code[offset + 1] as usize;
                let mid_byte = self.code[offset + 2] as usize;
                let high_byte = self.code[offset + 3] as usize;
                let constant_idx = (high_byte << 16) | (mid_byte << 8) | low_byte;
                let constant = self.constants.get(constant_idx).unwrap();
                println!(
                    "{:<16} {:>4} '{:?}'",
                    "OP_CONSTANT_LONG", constant_idx, constant
                );
                offset += 4;
            }
            Ok(OpCode::Negate) => {
                offset += 1;
                println!("OP_NEGATE");
            }
            Ok(OpCode::Add) => {
                offset += 1;
                println!("OP_ADD");
            }
            Ok(OpCode::Subtract) => {
                offset += 1;
                println!("OP_SUBTRACT");
            }
            Ok(OpCode::Multiply) => {
                offset += 1;
                println!("OP_MULTIPLY");
            }
            Ok(OpCode::Divide) => {
                offset += 1;
                println!("OP_DIVIDE");
            }
            Ok(OpCode::Return) => {
                offset += 1;
                println!("OP_RETURN");
            }
            Err(_) => {
                offset += 1;
                println!("Invalid Opcode '{}'", instruction);
            }
        };

        offset
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}
