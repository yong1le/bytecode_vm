use crate::{
    ast::{
        expr::{Expr, ExprVisitor},
        stmt::{Stmt, StmtVisitor},
    },
    chunk::Chunk,
    core::{
        errors::InterpretError,
        token::{Token, TokenType},
        value::{Object, Value},
    },
    heap::Heap,
    opcode::OpCode,
    parser::Parser,
};

type Return = Result<(), InterpretError>;

pub struct Compiler<'a> {
    statements: Parser<'a>,
    chunk: Chunk,
    heap: &'a mut Heap,
}

impl<'a> Compiler<'a> {
    pub fn new(statements: Parser<'a>, heap: &'a mut Heap) -> Self {
        Compiler {
            statements,
            heap,
            chunk: Chunk::new(),
        }
    }

    pub fn compile(mut self) -> Result<Chunk, InterpretError> {
        while let Some(stmt) = self.statements.next() {
            match stmt {
                Ok(stmt) => {
                    self.compile_stmt(&stmt)?;
                }
                Err(e) => {
                    return Err(e);
                }
            }
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

    /// Emits instruction `op`, that expects one operand. If the constant pool already
    /// exceeds 255 constants, this functions emits the long version of `op`, encoding
    /// the constant index as 3 operands.
    fn emit_constant_instruction(&mut self, op: OpCode, operand: Value, line: u32) {
        let constant_idx = self.chunk.add_constant(operand);

        if constant_idx > 255 {
            self.emit_byte(op.to_long() as u8, line);
            self.emit_byte((constant_idx & 255) as u8, line);
            self.emit_byte(((constant_idx >> 8) & 255) as u8, line);
            self.emit_byte(((constant_idx >> 16) & 255) as u8, line);
        } else {
            self.emit_byte(op as u8, line);
            self.emit_byte(constant_idx as u8, line);
        }
    }
}

impl StmtVisitor<Return> for Compiler<'_> {
    fn visit_print(&mut self, token: &Token, expr: &Expr) -> Return {
        self.compile_expr(expr)?;
        self.chunk.write_byte(OpCode::Print as u8, token.line);
        Ok(())
    }

    fn visit_expr(&mut self, token: &Token, expr: &Expr) -> Return {
        self.compile_expr(expr)?;
        self.chunk.write_byte(OpCode::Pop as u8, token.line);
        Ok(())
    }

    fn visit_declare_var(&mut self, id: &Token, expr: &Option<Expr>) -> Return {
        match expr {
            Some(expr) => self.compile_expr(expr)?,
            None => self.emit_constant_instruction(OpCode::Constant, Value::nil(), id.line),
        }

        let object = self.heap.push(Object::String(id.lexeme.to_string()));
        self.emit_constant_instruction(OpCode::DefineGlobal, object, id.line);

        Ok(())
    }

    fn visit_block(&mut self, statements: &[Stmt]) -> Return {
        todo!()
    }

    fn visit_if(
        &mut self,
        condition: &Expr,
        if_block: &Stmt,
        else_block: &Option<Box<Stmt>>,
    ) -> Return {
        todo!()
    }

    fn visit_while(&mut self, condition: &Expr, while_block: &Stmt) -> Return {
        todo!()
    }

    fn visit_declare_func(
        &mut self,
        id: &Token,
        params: &std::rc::Rc<Vec<Token>>,
        body: &std::rc::Rc<Vec<Stmt>>,
    ) -> Return {
        todo!()
    }

    fn visit_return(&mut self, token: &Token, expr: &Expr) -> Return {
        todo!()
    }

    fn visit_declare_class(
        &mut self,
        id: &Token,
        parent: &Option<Token>,
        methods: &[(Token, std::rc::Rc<Vec<Token>>, std::rc::Rc<Vec<Stmt>>)],
    ) -> Return {
        todo!()
    }
}

impl ExprVisitor<Return> for Compiler<'_> {
    fn visit_literal(&mut self, token: &Token) -> Return {
        match &token.token {
            TokenType::Number => {
                self.emit_constant_instruction(
                    OpCode::Constant,
                    Value::number(token.lexeme.parse().unwrap()),
                    token.line,
                );
            }
            TokenType::True => {
                self.emit_constant_instruction(OpCode::Constant, Value::boolean(true), token.line);
            }
            TokenType::False => {
                self.emit_constant_instruction(OpCode::Constant, Value::boolean(false), token.line);
            }
            TokenType::Nil => {
                self.emit_constant_instruction(OpCode::Constant, Value::nil(), token.line);
            }
            TokenType::String => {
                let object = Object::String(token.lexeme[1..token.lexeme.len() - 1].to_string());
                let object_idx = self.heap.push(object);
                self.emit_constant_instruction(OpCode::Constant, object_idx, token.line);
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
        let object = self.heap.push(Object::String(id.lexeme.to_string()));
        self.emit_constant_instruction(OpCode::GetGlobal, object, id.line);

        Ok(())
    }

    fn visit_assignment(&mut self, id: &Token, assignment: &Expr) -> Return {
        self.compile_expr(assignment)?;

        let object = self.heap.push(Object::String(id.lexeme.to_string()));
        self.emit_constant_instruction(OpCode::SetGlobal, object, id.line);

        Ok(())
    }

    fn visit_and(&mut self, left: &Expr, right: &Expr) -> Return {
        todo!()
    }

    fn visit_or(&mut self, left: &Expr, right: &Expr) -> Return {
        todo!()
    }

    fn visit_call(&mut self, callee: &Expr, arguments: &[Expr], closing: &Token) -> Return {
        todo!()
    }

    fn visit_get(&mut self, obj: &Expr, prop: &Token) -> Return {
        todo!()
    }

    fn visit_set(&mut self, obj: &Expr, prop: &Token, value: &Expr) -> Return {
        todo!()
    }

    fn visit_this(&mut self, token: &Token) -> Return {
        todo!()
    }

    fn visit_super(&mut self, super_token: &Token, prop: &Token) -> Return {
        todo!()
    }
}
