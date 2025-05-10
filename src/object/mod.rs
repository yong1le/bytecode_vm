mod functions;
pub mod native;

use std::rc::Rc;

pub use functions::Function;
use native::Native;

pub enum Object {
    String(String),
    Function(Rc<Function>),
    Native(Box<dyn Native>),
}
