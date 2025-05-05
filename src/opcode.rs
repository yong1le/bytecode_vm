use derive_more::TryFrom;

#[derive(Debug, TryFrom)]
#[try_from(repr)]
#[repr(u8)]
pub enum OpCode {
    Constant,
    ConstantLong,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Return,
}
