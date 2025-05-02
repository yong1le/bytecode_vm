use super::{callable::LoxCallable, literal::Literal};

#[derive(Clone, Debug)]
pub struct LoxClass {
    name: String,
}

impl LoxClass {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl LoxCallable for LoxClass {
    fn name(&self) -> &str {
        &self.name
    }

    fn call(
        &self,
        _: &mut crate::runtime::interpreter::Interpreter,
        _: Vec<super::literal::Literal>,
    ) -> Result<super::literal::Literal, super::errors::RuntimeError> {
        Ok(Literal::Instance(LoxInstance::new(self.name.to_string())))
    }

    fn arity(&self) -> usize {
        0
    }
}

#[derive(Clone, Debug)]
pub struct LoxInstance {
    name: String,
}

impl LoxInstance {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
