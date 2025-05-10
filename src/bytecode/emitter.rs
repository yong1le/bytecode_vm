use crate::core::{
    errors::{CompileError, InterpretError},
    OpCode, Value,
};

use super::{chunk::Chunk, Compiler, Return};

/// Implementation responsible for emitting bytecode to the chunk
impl Compiler<'_> {
    pub(crate) fn get_chunk(&mut self) -> &mut Chunk {
        &mut self.function.chunk
    }

    pub(crate) fn get_code_length(&self) -> usize {
        self.function.chunk.code.len()
    }
    /// Emits a single byte to the chunk
    pub(crate) fn emit_byte(&mut self, byte: u8, line: u32) {
        self.get_chunk().write_byte(byte, line);
    }

    /// Emits instruction `op` that expects one operand pointing to an index on the
    /// constants pool. If the operand does not point to the operand pool, use
    /// `emit_operand_instruction` instead.
    pub(crate) fn emit_constant_instruction(&mut self, op: OpCode, operand: Value, line: u32) {
        let constant_idx = self.get_chunk().add_constant(operand);

        self.emit_operand_instruction(op, constant_idx, line);
    }

    /// Emits instruction `op` that expects one operand `index`. If the operand exceeds
    /// u8 (255), this functions emit the long version of `op`, encoding the single `index`
    /// operand as 3 operands.
    pub(crate) fn emit_operand_instruction(&mut self, op: OpCode, index: usize, line: u32) {
        if index > 255 {
            self.emit_byte(op.to_long() as u8, line);
            self.emit_byte((index & 255) as u8, line);
            self.emit_byte(((index >> 8) & 255) as u8, line);
            self.emit_byte(((index >> 16) & 255) as u8, line);
        } else {
            self.emit_byte(op as u8, line);
            self.emit_byte(index as u8, line);
        }
    }

    /// Emits a jump instruction `op` and returns the index that the instruction was
    /// inserted at
    pub(crate) fn emit_jump_instruction(&mut self, op: OpCode, line: u32) -> usize {
        self.emit_byte(op as u8, line);
        // 2 byte operand for jumps
        self.emit_byte(OpCode::Nop as u8, line);
        self.emit_byte(OpCode::Nop as u8, line);

        self.get_code_length() - 2
    }

    /// Patches the jump distance
    pub(crate) fn patch_jump_instruction(&mut self, offset: usize, line: u32) -> Return {
        let code = &mut self.get_chunk().code;
        // -2 because our jump instruction has 2 operands
        let jump_distance = code.len() - offset - 2;

        if jump_distance > u16::MAX as usize {
            return Err(InterpretError::Compile(CompileError::LargeJump(
                line,
                jump_distance,
            )));
        };

        code[offset] = (jump_distance & 255) as u8;
        code[offset + 1] = ((jump_distance >> 8) & 255) as u8;

        Ok(())
    }

    pub(crate) fn emit_loop_instruction(&mut self, loop_start: usize, line: u32) -> Return {
        self.emit_byte(OpCode::Loop as u8, line);

        let jump_distance = self.get_code_length() - loop_start + 2;
        if jump_distance > u16::MAX as usize {
            return Err(InterpretError::Compile(CompileError::LargeJump(
                line,
                jump_distance,
            )));
        };

        self.emit_byte((jump_distance & 255) as u8, line);
        self.emit_byte(((jump_distance >> 8) & 255) as u8, line);

        Ok(())
    }
}
