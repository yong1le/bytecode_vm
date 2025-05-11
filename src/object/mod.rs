mod functions;
pub mod native;

use std::rc::Rc;

pub use functions::Function;
use native::Native;

pub enum Object {
    String(Rc<str>),
    Function(Rc<Function>),
    Native(Rc<dyn Native>),
}
