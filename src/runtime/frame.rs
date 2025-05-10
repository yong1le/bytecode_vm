use std::rc::Rc;

use crate::object::Function;

use super::VM;

#[derive(Debug)]
pub struct Frame {
    /// Index into a chunk's code
    pub ip: usize,
    /// Index into the VM's stack
    pub fp: usize,
    pub function: Rc<Function>,
}

impl Frame {
    pub fn new(function: Rc<Function>, fp: usize) -> Self {
        Self {
            ip: 0,
            fp,
            function,
        }
    }
}

impl VM<'_> {
    pub(crate) fn get_frame(&self) -> &Frame {
        let i = self.frames.len() - 1;
        &self.frames[i]
    }

    pub(crate) fn get_frame_mut(&mut self) -> &mut Frame {
        let i = self.frames.len() - 1;
        &mut self.frames[i]
    }
}
