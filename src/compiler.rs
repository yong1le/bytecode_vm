use crate::{
    ast::{
        expr::{Expr, ExprVisitor},
        stmt::{Stmt, StmtVisitor},
    },
    chunk::Chunk,
    core::{
        errors::{CompileError, InterpretError},
        token::{Token, TokenType},
        value::{Object, Value},
    },
    heap::Heap,
    opcode::OpCode,
    parser::Parser,
};

type Return = Result<(), InterpretError>;

struct Local {
    name: String,
    depth: usize,
    init: bool,
}

impl Local {
    pub fn new(name: String, depth: usize) -> Self {
        Self {
            name,
            depth,
            init: false,
        }
    }

    pub fn initialize(&mut self) {
        self.init = true;
    }
}

pub struct Compiler<'a> {
    statements: Parser<'a>,
    chunk: Chunk,
    heap: &'a mut Heap,
    /// The depth of nested scopes the compiler is currently in, 0 is the global scope
    scope_depth: usize,
    locals: Vec<Local>,
}

impl<'a> Compiler<'a> {
    pub fn new(statements: Parser<'a>, heap: &'a mut Heap) -> Self {
        Compiler {
            statements,
            heap,
            chunk: Chunk::new(),
            scope_depth: 0,
            locals: Vec::new(),
        }
    }

    /// Compiles the statements in the compiler into a chunk of bytecode to be used
    /// by the virtual machine. This function consumes the compiler instance.
    pub fn compile(mut self) -> Result<Chunk, Vec<InterpretError>> {
        let mut errors = vec![];

        while let Some(stmt) = self.statements.next() {
            match stmt {
                Ok(stmt) => {
                    if let Err(e) = self.compile_stmt(&stmt) {
                        errors.push(e);
                    }
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        self.chunk.write_byte(OpCode::Return as u8, 2);
        Ok(self.chunk)
    }

    fn compile_expr(&mut self, expression: &Expr) -> Return {
        expression.accept(self)
    }

    fn compile_stmt(&mut self, statement: &Stmt) -> Return {
        statement.accept(self)
    }

    /// Emits a single byte to the chunk
    fn emit_byte(&mut self, byte: u8, line: u32) {
        self.chunk.write_byte(byte, line);
    }

    /// Emits instruction `op` that expects one operand pointing to an index on the
    /// constants pool. If the operand does not point to the operand pool, use
    /// `emit_operand_instruction` instead.
    fn emit_constant_instruction(&mut self, op: OpCode, operand: Value, line: u32) {
        let constant_idx = self.chunk.add_constant(operand);

        self.emit_operand_instruction(op, constant_idx, line);
    }

    /// Emits instruction `op` that expects one operand `index`. If the operand exceeds
    /// u8 (255), this functions emit the long version of `op`, encoding the single `index`
    /// operand as 3 operands.
    fn emit_operand_instruction(&mut self, op: OpCode, index: usize, line: u32) {
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
    fn emit_jump_instruction(&mut self, op: OpCode, line: u32) -> usize {
        self.emit_byte(op as u8, line);
        // 2 byte operand for jumps
        self.emit_byte(OpCode::Nop as u8, line);
        self.emit_byte(OpCode::Nop as u8, line);

        self.chunk.code.len() - 2
    }

    /// Patches the jump distance
    fn patch_jump_instruction(&mut self, offset: usize, line: u32) -> Return {
        // -2 because our jump instruction has 2 operands
        let jump_distance = self.chunk.code.len() - offset - 2;

        if jump_distance > u16::MAX as usize {
            return Err(InterpretError::Compile(CompileError::LargeJump(
                line,
                jump_distance,
            )));
        };

        self.chunk.code[offset] = (jump_distance & 255) as u8;
        self.chunk.code[offset + 1] = ((jump_distance >> 8) & 255) as u8;

        Ok(())
    }

    fn emit_loop_instruction(&mut self, loop_start: usize, line: u32) -> Return {
        self.emit_byte(OpCode::Loop as u8, line);

        let jump_distance = self.chunk.code.len() - loop_start + 2;
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

    /// Declares a local variable `name` with the current scope depth, storing
    /// it into the internal locals array
    fn declare_local(&mut self, name: String, line: u32) -> Return {
        if self.scope_depth == 0 {
            return Ok(());
        }

        if self
            .locals
            .iter()
            .any(|l| l.depth == self.scope_depth && l.name == name)
        {
            return Err(InterpretError::Compile(CompileError::AlreadyDeclared(
                line, name,
            )));
        }

        self.locals.push(Local::new(name, self.scope_depth));

        Ok(())
    }

    fn define_local(&mut self) {
        let last = self.locals.len() - 1;
        self.locals[last].initialize();
    }

    fn resolve_local(&self, name: &str, line: u32) -> Result<Option<usize>, InterpretError> {
        match self.locals.iter().rposition(|l| l.name == *name) {
            None => Ok(None),
            Some(index) => {
                let local = self.locals.get(index).unwrap();
                if !local.init {
                    Err(InterpretError::Compile(CompileError::SelfInitialization(
                        line,
                    )))
                } else {
                    Ok(Some(index))
                }
            }
        }
    }
}

impl StmtVisitor<Return> for Compiler<'_> {
    fn visit_print(&mut self, token: &Token, expr: &Expr) -> Return {
        self.compile_expr(expr)?;
        self.emit_byte(OpCode::Print as u8, token.line);
        Ok(())
    }

    fn visit_expr(&mut self, token: &Token, expr: &Expr) -> Return {
        self.compile_expr(expr)?;
        self.emit_byte(OpCode::Pop as u8, token.line);
        Ok(())
    }

    fn visit_declare_var(&mut self, id: &Token, expr: &Option<Expr>) -> Return {
        if self.scope_depth > 0 {
            self.declare_local(id.lexeme.to_string(), id.line)?;
        }

        match expr {
            Some(expr) => self.compile_expr(expr)?,
            None => self.emit_constant_instruction(OpCode::LoadConstant, Value::nil(), id.line),
        }

        if self.scope_depth == 0 {
            let object = self.heap.push(Object::String(id.lexeme.to_string()));
            self.emit_constant_instruction(OpCode::DefineGlobal, object, id.line);
        } else {
            self.define_local();
        }
        Ok(())
    }

    fn visit_block(&mut self, statements: &[Stmt]) -> Return {
        self.scope_depth += 1;
        for stmt in statements {
            self.compile_stmt(stmt)?;
        }
        self.scope_depth -= 1;

        // Remove all local variables from that block
        let mut to_remove = 0;
        self.locals.retain(|l| {
            if l.depth > self.scope_depth {
                to_remove += 1;
                false
            } else {
                true
            }
        });
        for _ in 0..to_remove {
            self.emit_byte(OpCode::Pop as u8, 0);
        }
        Ok(())
    }

    fn visit_if(
        &mut self,
        token: &Token,
        condition: &Expr,
        if_block: &Stmt,
        else_block: &Option<Box<Stmt>>,
    ) -> Return {
        self.compile_expr(condition)?;

        let if_offset = self.emit_jump_instruction(OpCode::JumpIfFalse, token.line);
        self.emit_byte(OpCode::Pop as u8, token.line); // removes condition value off stack
        self.compile_stmt(if_block)?;

        // send JUMP here to include it inside the if_block
        let else_offset = self.emit_jump_instruction(OpCode::Jump, token.line);
        self.emit_byte(OpCode::Pop as u8, token.line); // removes condition value off stack

        self.patch_jump_instruction(if_offset, token.line)?;

        if let Some(else_block) = else_block {
            self.compile_stmt(else_block)?;
        }
        self.patch_jump_instruction(else_offset, token.line)?;
        Ok(())
    }

    fn visit_while(&mut self, token: &Token, condition: &Expr, while_block: &Stmt) -> Return {
        let loop_start = self.chunk.code.len();

        self.compile_expr(condition)?;
        let offset = self.emit_jump_instruction(OpCode::JumpIfFalse, token.line);
        self.emit_byte(OpCode::Pop as u8, token.line); // removes condition value off stack

        self.compile_stmt(while_block)?;
        self.emit_loop_instruction(loop_start, token.line)?;
        self.patch_jump_instruction(offset, token.line)?;
        // removes condition value off stack, even if we skipped the loop body
        self.emit_byte(OpCode::Pop as u8, token.line);

        Ok(())
    }

    fn visit_declare_func(
        &mut self,
        id: &Token,
        params: &std::rc::Rc<Vec<Token>>,
        body: &std::rc::Rc<Vec<Stmt>>,
    ) -> Return {
        Err(InterpretError::UnImplemented)
    }

    fn visit_return(&mut self, token: &Token, expr: &Expr) -> Return {
        Err(InterpretError::UnImplemented)
    }

    fn visit_declare_class(
        &mut self,
        id: &Token,
        parent: &Option<Token>,
        methods: &[(Token, std::rc::Rc<Vec<Token>>, std::rc::Rc<Vec<Stmt>>)],
    ) -> Return {
        Err(InterpretError::UnImplemented)
    }
}

impl ExprVisitor<Return> for Compiler<'_> {
    fn visit_literal(&mut self, token: &Token) -> Return {
        match &token.token {
            TokenType::Number => {
                self.emit_constant_instruction(
                    OpCode::LoadConstant,
                    Value::number(token.lexeme.parse().unwrap()),
                    token.line,
                );
            }
            TokenType::True => {
                self.emit_constant_instruction(
                    OpCode::LoadConstant,
                    Value::boolean(true),
                    token.line,
                );
            }
            TokenType::False => {
                self.emit_constant_instruction(
                    OpCode::LoadConstant,
                    Value::boolean(false),
                    token.line,
                );
            }
            TokenType::Nil => {
                self.emit_constant_instruction(OpCode::LoadConstant, Value::nil(), token.line);
            }
            TokenType::String => {
                let object = Object::String(token.lexeme[1..token.lexeme.len() - 1].to_string());
                let object_idx = self.heap.push(object);
                self.emit_constant_instruction(OpCode::LoadConstant, object_idx, token.line);
            }
            _ => panic!(
                "PANIC: Invalid Token {:?} passed to <compiler.visit_binary>",
                token.token
            ),
        }
        Ok(())
    }

    fn visit_unary(&mut self, operator: &Token, expr: &Expr) -> Return {
        match operator.token {
            TokenType::Minus => {
                self.compile_expr(expr)?;
                self.emit_byte(OpCode::Negate as u8, operator.line);
            }
            TokenType::Bang => {
                self.compile_expr(expr)?;
                self.emit_byte(OpCode::Not as u8, operator.line);
            }
            _ => panic!(
                "PANIC: Invalid Token {:?} passed to <compiler.visit_unary>",
                operator.token
            ),
        }

        Ok(())
    }

    fn visit_binary(&mut self, operator: &Token, left: &Expr, right: &Expr) -> Return {
        let opcode = match operator.token {
            TokenType::Plus => OpCode::Add,
            TokenType::Minus => OpCode::Subtract,
            TokenType::Star => OpCode::Multiply,
            TokenType::Slash => OpCode::Divide,
            TokenType::EqualEqual => OpCode::Equal,
            TokenType::BangEqual => OpCode::NotEqual,
            TokenType::LessThan => OpCode::LessThan,
            TokenType::LessEqual => OpCode::LessEqual,
            TokenType::GreaterThan => OpCode::GreaterThan,
            TokenType::GreaterEqual => OpCode::GreaterEqual,
            _ => panic!(
                "PANIC: Invalid Token {:?} passed to <compiler.visit_binary>",
                operator.token
            ),
        };

        self.compile_expr(left)?;
        self.compile_expr(right)?;
        self.emit_byte(opcode as u8, operator.line);

        Ok(())
    }

    fn visit_grouping(&mut self, expr: &Expr) -> Return {
        self.compile_expr(expr)
    }

    fn visit_variable(&mut self, id: &Token) -> Return {
        match self.resolve_local(&id.lexeme, id.line)? {
            Some(index) => {
                self.emit_operand_instruction(OpCode::GetLocal, index, id.line);
            }
            None => {
                let object_idx = self.heap.push(Object::String(id.lexeme.to_string()));
                self.emit_constant_instruction(OpCode::GetGlobal, object_idx, id.line);
            }
        }

        Ok(())
    }

    fn visit_assignment(&mut self, id: &Token, assignment: &Expr) -> Return {
        self.compile_expr(assignment)?;

        match self.resolve_local(&id.lexeme, id.line)? {
            Some(index) => {
                self.emit_operand_instruction(OpCode::SetLocal, index, id.line);
            }
            None => {
                let object = self.heap.push(Object::String(id.lexeme.to_string()));
                self.emit_constant_instruction(OpCode::SetGlobal, object, id.line);
            }
        }

        Ok(())
    }

    // Returns first false, or last value
    fn visit_and(&mut self, token: &Token, left: &Expr, right: &Expr) -> Return {
        self.compile_expr(left)?;
        let end_offset = self.emit_jump_instruction(OpCode::JumpIfFalse, token.line);
        self.emit_byte(OpCode::Pop as u8, token.line);
        self.compile_expr(right)?;
        self.patch_jump_instruction(end_offset, token.line)?;

        Ok(())
    }

    // Returns first true, or last value
    fn visit_or(&mut self, token: &Token, left: &Expr, right: &Expr) -> Return {
        self.compile_expr(left)?;
        let else_offset = self.emit_jump_instruction(OpCode::JumpIfFalse, token.line);
        let end_offset = self.emit_jump_instruction(OpCode::Jump, token.line);

        // left == false, jump past the end jump, and go to the right expr
        // left == true, visit the end jump instruction, which jumps to the end, skipping right
        self.patch_jump_instruction(else_offset, token.line)?;
        self.emit_byte(OpCode::Pop as u8, token.line);

        self.compile_expr(right)?;
        self.patch_jump_instruction(end_offset, token.line)?;

        Ok(())
    }

    fn visit_call(&mut self, callee: &Expr, arguments: &[Expr], closing: &Token) -> Return {
        Err(InterpretError::UnImplemented)
    }

    fn visit_get(&mut self, obj: &Expr, prop: &Token) -> Return {
        Err(InterpretError::UnImplemented)
    }

    fn visit_set(&mut self, obj: &Expr, prop: &Token, value: &Expr) -> Return {
        Err(InterpretError::UnImplemented)
    }

    fn visit_this(&mut self, token: &Token) -> Return {
        Err(InterpretError::UnImplemented)
    }

    fn visit_super(&mut self, super_token: &Token, prop: &Token) -> Return {
        Err(InterpretError::UnImplemented)
    }
}
