use thiserror::Error;

use crate::{
    ast::{
        expr::{Expr, ExprVisitor},
        stmt::{Stmt, StmtVisitor},
    },
    chunk::Chunk,
    core::{
        token::{Token, TokenType},
        value::{Object, Value},
    },
    opcode::OpCode,
    parser::Parser,
    vm::VM,
};

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("Compile Error")]
    Error,
}

type Return = Result<(), CompileError>;

pub struct Compiler<'a> {
    statements: Parser<'a>,
    chunk: Chunk,
    vm: &'a mut VM,
}

impl<'a> Compiler<'a> {
    pub fn new(statements: Parser<'a>, vm: &'a mut VM) -> Self {
        Compiler {
            statements,
            vm,
            chunk: Chunk::new(),
        }
    }

    pub fn compile(&mut self) -> Result<Chunk, CompileError> {
        self.chunk = Chunk::new();
        while let Some(expr) = self.statements.next() {
            match expr {
                Ok(expr) => {
                    self.compile_expr(&expr)?;
                }
                Err(_) => {
                    return Err(CompileError::Error);
                }
            }
        }

        self.chunk.write_byte(OpCode::Return as u8, 2);
        Ok(self.chunk.clone())
    }

    fn compile_expr(&mut self, expression: &Expr) -> Return {
        expression.accept(self)
    }

    fn compile_stmt(&mut self, statement: &Stmt) -> Return {
        statement.accept(self)
    }
}

impl StmtVisitor<Return> for Compiler<'_> {
    fn visit_print(&mut self, stmt: &Expr) -> Return {
        todo!()
    }

    fn visit_expr(&mut self, expr: &Expr) -> Return {
        todo!()
    }

    fn visit_declare_var(&mut self, id: &Token, expr: &Option<Expr>) -> Return {
        todo!()
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

    fn visit_return(&mut self, expr: &Expr, line: &u32) -> Return {
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
                self.chunk
                    .write_constant(Value::number(token.lexeme.parse().unwrap()), token.line);
            }
            TokenType::True => {
                self.chunk.write_constant(Value::boolean(true), token.line);
            }
            TokenType::False => {
                self.chunk.write_constant(Value::boolean(false), token.line);
            }
            TokenType::Nil => {
                self.chunk.write_constant(Value::nil(), token.line);
            }
            TokenType::String => {
                let object = Object::String(token.lexeme[1..token.lexeme.len() - 1].to_string());
                self.chunk.write_constant(self.vm.alloc(object), token.line);
            }
            t => panic!("No such token type: {:?}", t),
        }
        Ok(())
    }

    fn visit_unary(&mut self, operator: &Token, expr: &Expr) -> Return {
        match operator.token {
            TokenType::Minus => {
                self.compile_expr(expr)?;
                self.chunk.write_byte(OpCode::Negate as u8, operator.line);
            }
            _ => todo!(),
        }

        Ok(())
    }

    fn visit_binary(&mut self, operator: &Token, left: &Expr, right: &Expr) -> Return {
        let opcode = match operator.token {
            TokenType::Plus => OpCode::Add,
            TokenType::Minus => OpCode::Subtract,
            TokenType::Star => OpCode::Multiply,
            TokenType::Slash => OpCode::Divide,
            _ => todo!(),
        };

        self.compile_expr(left)?;
        self.compile_expr(right)?;
        self.chunk.write_byte(opcode as u8, operator.line);

        Ok(())
    }

    fn visit_grouping(&mut self, expr: &Expr) -> Return {
        self.compile_expr(expr)
    }

    fn visit_variable(&mut self, id: &Token) -> Return {
        todo!()
    }

    fn visit_assignment(&mut self, id: &Token, assignment: &Expr) -> Return {
        todo!()
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
