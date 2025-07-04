mod chunk;
mod compiler;
mod emitter;
mod locals;

pub use chunk::Chunk;

use crate::{
    ast::{expr::Expr, stmt::Stmt},
    core::{errors::InterpretError, OpCode},
    frontend::Parser,
    object::Function,
    runtime::{Heap, FRAME_MAX},
};
use locals::{CompilerUpvalue, Local};

type Return = Result<(), InterpretError>;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
enum FunctionType {
    Main,
    Function,
}

pub struct Compiler<'a> {
    statements: Parser<'a>,
    function_type: FunctionType,
    function: Function,
    heap: Option<&'a mut Heap>,
    /// The depth of nested scopes the compiler is currently in, 0 is the global scope
    scope_depth: usize,
    locals: Vec<Local>,
    upvalues: Vec<CompilerUpvalue>,
    enclosing: Option<*mut Self>,
}

impl<'a> Compiler<'a> {
    pub fn new(statements: Parser<'a>, heap: &'a mut Heap) -> Self {
        Compiler {
            statements,
            heap: Some(heap),
            function: Function::new("main".to_string(), 0),
            scope_depth: 0,
            locals: vec![Local::new("".to_string(), 0)],
            function_type: FunctionType::Main,
            upvalues: Vec::with_capacity(FRAME_MAX),
            enclosing: None,
        }
    }

    /// Compiles the statements in the compiler into a chunk of bytecode to be used
    /// by the virtual machine. This function consumes the compiler instance.
    pub fn compile(mut self) -> Result<Function, Vec<InterpretError>> {
        let mut errors = vec![];

        while let Some(stmt) = self.statements.next() {
            match stmt {
                Ok(stmt) => {
                    if let Err(e) = self.compile_stmt(stmt) {
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

        self.emit_byte(OpCode::Return as u8, 2);
        Ok(self.function)
    }

    fn compile_expr(&mut self, expression: Expr) -> Return {
        expression.accept(self)
    }

    fn compile_stmt(&mut self, statement: Stmt) -> Return {
        statement.accept(self)
    }
}
