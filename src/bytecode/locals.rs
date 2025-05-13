use crate::core::{
    errors::{CompileError, InterpretError},
    OpCode,
};

use super::{Compiler, Return};

pub struct Local {
    name: String,
    depth: usize,
    init: bool,
    is_captured: bool,
}

pub struct CompilerUpvalue {
    pub(crate) index: usize,
    pub(crate) is_local: bool,
}

impl Local {
    pub fn new(name: String, depth: usize) -> Self {
        Self {
            name,
            depth,
            init: false,
            is_captured: false,
        }
    }

    pub fn initialize(&mut self) {
        self.init = true;
    }

    pub fn capture(&mut self) {
        self.is_captured = true;
    }
}

impl Compiler<'_> {
    pub(crate) fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    pub(crate) fn end_scope(&mut self) {
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
    }

    /// Declares a local variable `name` with the current scope depth, storing
    /// it into the internal locals array
    pub(crate) fn declare_local(&mut self, name: String, line: u32) -> Return {
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

    pub(crate) fn define_local(&mut self) {
        if self.scope_depth == 0 {
            return;
        }

        let last = self.locals.len() - 1;
        self.locals[last].initialize();
    }

    pub(crate) fn resolve_local(
        &self,
        name: &str,
        line: u32,
    ) -> Result<Option<usize>, InterpretError> {
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

    pub(crate) fn resolve_upvalue(
        &mut self,
        name: &str,
        line: u32,
    ) -> Result<Option<usize>, InterpretError> {
        match self.enclosing {
            None => Ok(None),
            Some(enclosing) => {
                let local = unsafe { (*enclosing).resolve_local(name, line)? };
                match local {
                    Some(stack_index) => Ok(Some(self.add_upvalue(stack_index, true))),
                    None => {
                        let upvalue = unsafe { (*enclosing).resolve_upvalue(name, line) }?;
                        match upvalue {
                            Some(stack_index) => Ok(Some(self.add_upvalue(stack_index, true))),
                            None => Ok(None),
                        }
                    }
                }
            }
        }
    }

    fn add_upvalue(&mut self, index: usize, is_local: bool) -> usize {
        self.upvalues.push(CompilerUpvalue { index, is_local });
        self.function.upvalue_count += 1;

        self.function.upvalue_count - 1
    }
}
