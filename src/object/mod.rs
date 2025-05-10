mod functions;

use std::rc::Rc;

pub use functions::Function;

#[derive(Debug)]
pub enum Object {
    String(String),
    Function(Rc<Function>),
}
