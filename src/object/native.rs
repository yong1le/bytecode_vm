use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::{errors::RuntimeError, Value};

pub trait Native {
    fn name(&self) -> &str;
    fn arity(&self) -> u8;
    fn call(&self, args: Vec<Value>) -> Result<Value, RuntimeError>;
}

pub struct Clock;
impl Native for Clock {
    fn name(&self) -> &str {
        "clock"
    }

    fn arity(&self) -> u8 {
        0
    }

    fn call(&self, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards.");

        Ok(Value::number(time.as_secs_f64().trunc()))
    }
}

pub struct Sqrt;
impl Native for Sqrt {
    fn name(&self) -> &str {
        "sqrt"
    }

    fn arity(&self) -> u8 {
        1
    }

    fn call(&self, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let arg = args[0];

        if arg.is_number() {
            Ok(Value::number(f64::sqrt(arg.as_number())))
        } else {
            Err(RuntimeError::OperandMismatch(0, "number".to_string()))
        }
    }
}
