mod closure;
mod functions;

pub mod native;

use std::rc::Rc;

pub use closure::Closure;
pub use functions::Function;
use native::Native;

use crate::core::Value;

pub enum Object {
    String(Rc<str>),
    Function(Rc<Function>),
    Native(Rc<dyn Native>),
    Closure(Rc<Closure>),
    UpValue(Value),
}
