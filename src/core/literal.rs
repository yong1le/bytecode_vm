use std::{borrow::Cow, cell::RefCell, fmt, rc::Rc};

use super::{
    callable::LoxCallable,
    class::{LoxClass, LoxInstance},
};

/// The literal values that can be used by Lox.
#[derive(Debug, Clone)]
pub enum Literal {
    String(Cow<'static, str>),
    Number(f64),
    Boolean(bool),
    Callable(Rc<dyn LoxCallable>),
    Class(LoxClass),
    Instance(Rc<RefCell<LoxInstance>>),
    Nil,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Literal::String(a) => a.to_string(),
                Literal::Number(a) => {
                    if a.fract() == 0.0 {
                        format!("{:.1}", a)
                    } else {
                        format!("{}", a)
                    }
                }
                Literal::Boolean(a) => a.to_string(),
                Literal::Callable(c) => format!("<fn {}>", c.name()),
                Literal::Class(c) => c.name().to_string(),
                Literal::Instance(c) => format!("{} instance", c.borrow().name()),
                Literal::Nil => "nil".to_string(),
            }
        )
    }
}

impl Literal {
    /// For printing to the terminal, different rules from scan phase
    pub fn stringify(&self) -> String {
        match self {
            Literal::Number(n) => {
                format!("{n}")
            }
            _ => format!("{self}"),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Literal::Boolean(b) => b.to_owned(),
            Literal::Nil => false,
            _ => true,
        }
    }
}
