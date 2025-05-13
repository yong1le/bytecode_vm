mod closure;
mod functions;
mod upvalue;

pub mod native;

use std::{cell::RefCell, rc::Rc};

pub use closure::Closure;
pub use functions::Function;
use native::Native;
pub use upvalue::VMUpvalue;

use crate::core::Value;

pub enum Object {
    String(Rc<str>),
    Function(Rc<Function>),
    Native(Rc<dyn Native>),
    Closure(Rc<Closure>),
    UpValue(Rc<RefCell<Value>>),
}
