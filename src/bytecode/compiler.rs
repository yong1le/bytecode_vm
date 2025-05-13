use std::{
    mem::{self, MaybeUninit},
    rc::Rc,
};

use crate::{
    ast::{
        expr::{Expr, ExprVisitor},
        stmt::{Stmt, StmtVisitor},
    },
    core::{
        errors::{CompileError, InterpretError, PanicError},
        token::{Token, TokenType},
        OpCode, Value,
    },
    frontend::{Parser, Scanner},
    object::{Function, Object},
};

use super::{Compiler, FunctionType, Return};

impl StmtVisitor<Return> for Compiler<'_> {
    fn visit_print(&mut self, token: Token, expr: Expr) -> Return {
        self.compile_expr(expr)?;
        self.emit_byte(OpCode::Print as u8, token.line);
        Ok(())
    }

    fn visit_expr(&mut self, token: Token, expr: Expr) -> Return {
        self.compile_expr(expr)?;
        self.emit_byte(OpCode::Pop as u8, token.line);
        Ok(())
    }

    fn visit_declare_var(&mut self, id: Token, expr: Option<Expr>) -> Return {
        self.declare_local(id.lexeme.clone(), id.line)?;

        match expr {
            Some(expr) => self.compile_expr(expr)?,
            None => self.emit_constant_instruction(OpCode::LoadConstant, Value::nil(), id.line),
        }

        if self.scope_depth == 0 {
            let object = self.heap.as_mut().unwrap().push_str(id.lexeme);
            self.emit_constant_instruction(OpCode::DefineGlobal, object, id.line);
        }

        self.define_local();
        Ok(())
    }

    fn visit_block(&mut self, statements: Vec<Stmt>) -> Return {
        self.begin_scope();
        for stmt in statements {
            self.compile_stmt(stmt)?;
        }
        self.end_scope();

        Ok(())
    }

    fn visit_if(
        &mut self,
        token: Token,
        condition: Expr,
        if_block: Stmt,
        else_block: Option<Box<Stmt>>,
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
            self.compile_stmt(*else_block)?;
        }
        self.patch_jump_instruction(else_offset, token.line)?;
        Ok(())
    }

    fn visit_while(&mut self, token: Token, condition: Expr, while_block: Stmt) -> Return {
        let loop_start = self.get_code_length();

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

    fn visit_declare_func(&mut self, id: Token, params: Vec<Token>, body: Vec<Stmt>) -> Return {
        self.declare_local(id.lexeme.clone(), id.line)?;
        self.define_local();

        // Now, self.heap is None, and if we try to access it, we will get panic error. In general,
        // any compiler code should not access enclosing.heap
        let heap = self.heap.take();
        let mut new_compiler = Compiler {
            statements: Parser::new(Scanner::new("")), // placeholder, never actually used
            heap,
            function: Function::new(id.lexeme.clone(), params.len() as u8),
            scope_depth: 1,
            locals: vec![],
            function_type: FunctionType::Function,
            upvalues: Vec::new(),
            enclosing: Some(self as *mut Self), // should usually be safe, since we create and
        };

        // This block is reserved for operations that new_compiler does, we should never touch
        // `self` in this block manually
        {
            // [ <fn> ] [ arg1 ] [ arg2 ]
            new_compiler.declare_local(id.lexeme.clone(), id.line)?;
            new_compiler.define_local();
            for param in params {
                new_compiler.declare_local(param.lexeme, param.line)?;
                new_compiler.define_local();
            }
            for stmt in body {
                new_compiler.compile_stmt(stmt)?;
            }

            // Default 'return nil'. Frame exists at first return, so it will not run if there
            // is already a return in the function
            new_compiler.emit_constant_instruction(OpCode::LoadConstant, Value::nil(), id.line);
            new_compiler.emit_byte(OpCode::Return as u8, id.line);
        }

        self.heap = new_compiler.heap.take(); // take back our original heap
        let upvalues = mem::take(&mut new_compiler.upvalues);
        let new_function = new_compiler.function; // get the compiled function

        if upvalues.len() > 256 {
            panic!("Cannot have more than 256 upvalues in a closure.")
        }

        let function_idx = self
            .heap
            .as_mut()
            .unwrap()
            .push(Object::Function(Rc::new(new_function)));
        self.emit_operand_instruction(OpCode::Closure, function_idx.as_object(), id.line);

        for upvalue in upvalues {
            self.emit_byte(if upvalue.is_local { 1 } else { 0 } as u8, id.line);
            self.emit_byte(upvalue.index as u8, id.line);
        }

        if self.scope_depth == 0 {
            let function_name_idx = self.heap.as_mut().unwrap().push_str(id.lexeme);
            self.emit_constant_instruction(OpCode::DefineGlobal, function_name_idx, id.line);
        }

        Ok(())
    }

    fn visit_return(&mut self, token: Token, expr: Expr) -> Return {
        if self.function_type == FunctionType::Main {
            return Err(InterpretError::Compile(CompileError::TopReturn(token.line)));
        }

        self.compile_expr(expr)?;
        self.emit_byte(OpCode::Return as u8, token.line);
        Ok(())
    }

    fn visit_declare_class(
        &mut self,
        id: Token,
        parent: Option<Token>,
        methods: Vec<(Token, Vec<Token>, Vec<Stmt>)>,
    ) -> Return {
        Err(InterpretError::UnImplemented)
    }
}

impl ExprVisitor<Return> for Compiler<'_> {
    fn visit_literal(&mut self, token: Token) -> Return {
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
                let object_idx = self
                    .heap
                    .as_mut()
                    .unwrap()
                    .push_str(token.lexeme.replace("\"", ""));
                self.emit_constant_instruction(OpCode::LoadConstant, object_idx, token.line);
            }
            _ => {
                return Err(InterpretError::Panic(PanicError::InvalidToken(
                    token.line,
                    token.token,
                    "<compiler.visit_literal>".to_string(),
                )))
            }
        }
        Ok(())
    }

    fn visit_unary(&mut self, operator: Token, expr: Expr) -> Return {
        match operator.token {
            TokenType::Minus => {
                self.compile_expr(expr)?;
                self.emit_byte(OpCode::Negate as u8, operator.line);
            }
            TokenType::Bang => {
                self.compile_expr(expr)?;
                self.emit_byte(OpCode::Not as u8, operator.line);
            }
            _ => {
                return Err(InterpretError::Panic(PanicError::InvalidToken(
                    operator.line,
                    operator.token,
                    "<compiler.visit_unary>".to_string(),
                )))
            }
        }

        Ok(())
    }

    fn visit_binary(&mut self, operator: Token, left: Expr, right: Expr) -> Return {
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
            _ => {
                return Err(InterpretError::Panic(PanicError::InvalidToken(
                    operator.line,
                    operator.token,
                    "<compiler.visit_binary>".to_string(),
                )))
            }
        };

        self.compile_expr(left)?;
        self.compile_expr(right)?;
        self.emit_byte(opcode as u8, operator.line);

        Ok(())
    }

    fn visit_grouping(&mut self, expr: Expr) -> Return {
        self.compile_expr(expr)
    }

    fn visit_variable(&mut self, id: Token) -> Return {
        if let Some(index) = self.resolve_local(&id.lexeme, id.line)? {
            self.emit_operand_instruction(OpCode::GetLocal, index, id.line);
        } else if let Some(index) = self.resolve_upvalue(&id.lexeme, id.line)? {
            self.emit_operand_instruction(OpCode::GetUpvalue, index, id.line);
        } else {
            let variable_idx = self.heap.as_mut().unwrap().push_str(id.lexeme);
            self.emit_constant_instruction(OpCode::GetGlobal, variable_idx, id.line);
        }

        Ok(())
    }

    fn visit_assignment(&mut self, id: Token, assignment: Expr) -> Return {
        self.compile_expr(assignment)?;

        if let Some(index) = self.resolve_local(&id.lexeme, id.line)? {
            self.emit_operand_instruction(OpCode::SetLocal, index, id.line);
        } else if let Some(index) = self.resolve_upvalue(&id.lexeme, id.line)? {
            self.emit_operand_instruction(OpCode::SetUpvalue, index, id.line);
        } else {
            let object = self.heap.as_mut().unwrap().push_str(id.lexeme);
            self.emit_constant_instruction(OpCode::SetGlobal, object, id.line);
        }

        Ok(())
    }

    // Returns first false, or last value
    fn visit_and(&mut self, token: Token, left: Expr, right: Expr) -> Return {
        self.compile_expr(left)?;
        let end_offset = self.emit_jump_instruction(OpCode::JumpIfFalse, token.line);
        self.emit_byte(OpCode::Pop as u8, token.line);
        self.compile_expr(right)?;
        self.patch_jump_instruction(end_offset, token.line)?;

        Ok(())
    }

    // Returns first true, or last value
    fn visit_or(&mut self, token: Token, left: Expr, right: Expr) -> Return {
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

    fn visit_call(&mut self, callee: Expr, arguments: Vec<Expr>, closing: Token) -> Return {
        let argc = arguments.len();

        self.compile_expr(callee)?;
        for arg in arguments {
            self.compile_expr(arg)?;
        }

        self.emit_operand_instruction(OpCode::Call, argc, closing.line);
        Ok(())
    }

    fn visit_get(&mut self, obj: Expr, prop: Token) -> Return {
        Err(InterpretError::UnImplemented)
    }

    fn visit_set(&mut self, obj: Expr, prop: Token, value: Expr) -> Return {
        Err(InterpretError::UnImplemented)
    }

    fn visit_this(&mut self, token: Token) -> Return {
        Err(InterpretError::UnImplemented)
    }

    fn visit_super(&mut self, super_token: Token, prop: Token) -> Return {
        Err(InterpretError::UnImplemented)
    }
}
