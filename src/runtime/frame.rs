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
    #[inline]
    pub(crate) fn get_frame(&self) -> &Frame {
        let i = self.frames.len() - 1;
        &self.frames[i]
    }

    #[inline]
    pub(crate) fn get_frame_mut(&mut self) -> &mut Frame {
        let i = self.frames.len() - 1;
        &mut self.frames[i]
    }

    #[inline]
    pub(crate) fn push_frame(&mut self, frame: Frame) {
        if self.frames.len() > FRAME_MAX {
            panic!("STACK OVERFLOW");
        }
        self.frames.push(frame);
    }

    #[inline]
    pub(crate) fn pop_frame(&mut self) -> Frame {
        self.frames.pop().unwrap()
    }
}
