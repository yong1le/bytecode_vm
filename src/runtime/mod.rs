mod frame;
mod heap;
mod stack;
mod upvalue;
mod vm;

pub use frame::Frame;
pub use heap::Heap;
use rustc_hash::FxHashMap;
use slab::Slab;
use upvalue::VMUpvalue;

use crate::core::{errors::InterpretError, Value};
use std::io::Write;

type Return = Result<(), InterpretError>;

pub const FRAME_MAX: usize = 64;
pub const STACK_MAX: usize = 256;

pub struct VM<'a> {
    frame: Frame,
    frame_count: usize,
    stack: Vec<Value>,
    heap: Heap,
    globals: FxHashMap<u64, Value>,
    upvalues: Slab<VMUpvalue>,
    writer: Box<dyn Write + 'a>,
}
