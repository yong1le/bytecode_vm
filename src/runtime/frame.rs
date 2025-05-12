use std::rc::Rc;

use crate::object::Closure;

// TODO: Allocate frames from continuous memory
#[derive(Debug)]
pub struct Frame {
    /// Index into a chunk's code
    pub ip: usize,
    /// Index into the VM's stack
    pub fp: usize,
    pub closure: Rc<Closure>,

    pub caller: Option<Box<Frame>>,
}

impl Frame {
    pub fn new(closure: Rc<Closure>, fp: usize) -> Self {
        Self {
            ip: 0,
            fp,
            closure,
            caller: None,
        }
    }

    pub fn with_caller(closure: Rc<Closure>, fp: usize, caller: Box<Frame>) -> Self {
        Self {
            ip: 0,
            fp,
            closure,
            caller: Some(caller),
        }
    }
}
