use std::rc::Rc;

use crate::object::Function;

use super::{FRAME_MAX, VM};

#[derive(Debug)]
pub struct Frame {
    /// Index into a chunk's code
    pub ip: usize,
    /// Index into the VM's stack
    pub fp: usize,
    pub function: Rc<Function>,

    pub caller: Option<Box<Frame>>,
}

impl Frame {
    pub fn new(function: Rc<Function>, fp: usize) -> Self {
        Self {
            ip: 0,
            fp,
            function,
            caller: None,
        }
    }

    pub fn with_caller(function: Rc<Function>, fp: usize, caller: Box<Frame>) -> Self {
        Self {
            ip: 0,
            fp,
            function,
            caller: Some(caller),
        }
    }
}
