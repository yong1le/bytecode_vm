use core::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{errors::RuntimeError, literal::Literal};

pub trait LoxCallable: fmt::Debug {
    fn call(&self, arguments: Vec<Literal>) -> Result<Literal, RuntimeError>;
    fn arity(&self) -> usize;
    fn name(&self) -> &str;
}

// Native Functions
#[derive(Debug)]
pub struct Clock;
impl LoxCallable for Clock {
    fn call(&self, _: Vec<Literal>) -> Result<Literal, RuntimeError> {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards.");

        Ok(Literal::Number(time.as_secs_f64().trunc()))
    }

    fn arity(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "clock"
    }
}
