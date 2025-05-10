mod frame;
mod heap;
mod stack;
mod vm;

pub use frame::Frame;
pub use heap::Heap;
use rustc_hash::FxHashMap;

use crate::core::{errors::InterpretError, Value};
use std::io::Write;

type Return = Result<(), InterpretError>;

const FRAME_MAX: usize = 64;
const STACK_MAX: usize = 256;

pub struct VM<'a> {
    frames: Vec<Frame>,
    stack: Vec<Value>,
    heap: Heap,
    globals: FxHashMap<u64, Value>,
    writer: Box<dyn Write + 'a>,
}
