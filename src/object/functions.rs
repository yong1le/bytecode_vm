use crate::bytecode::Chunk;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum FunctionType {
    Main,
    Function,
}

pub struct Function {
    pub name: String,
    pub arity: u8,
    pub chunk: Chunk,
}

impl std::fmt::Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name)
    }
}

impl Function {
    pub fn new(name: String, arity: u8) -> Self {
        Self {
            name,
            arity,
            chunk: Chunk::new(),
        }
    }
}
