use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::Value;

pub trait Native {
    fn name(&self) -> &str;
    fn arity(&self) -> u8;
    fn call(&self, args: Vec<Value>) -> Value;
}

pub struct Clock;
impl Native for Clock {
    fn name(&self) -> &str {
        "clock"
    }

    fn arity(&self) -> u8 {
        0
    }

    fn call(&self, args: Vec<Value>) -> Value {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards.");

        Value::number(time.as_secs_f64().trunc())
    }
}
