use derive_more::TryFrom;

#[derive(Debug, TryFrom, Clone, Copy)]
#[try_from(repr)]
#[repr(u8)]
pub enum OpCode {
    /// Loads a constant from the constant pool onto the stack.
    ///
    /// ### Operand
    /// - 1 byte: index into the constant pool
    /// - 3 bytes: index into the constant pool (index > 255)
    ///
    /// ### Stack effect
    /// - Before: `[]`
    /// - After: `[value]`
    LoadConstant,
    /// Long version of [`OpCode::LoadConstantLong`]
    LoadConstantLong,

    /// Negates the value on top of the stack.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: `[value]`
    /// - After: `[-value]`
    Negate,

    /// Applies logical NOT to the value on top of the stack.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: `[value]`
    /// - After: `[!value]`
    Not,

    /// Adds the top two values on the stack.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: `[b, a]` TOP
    /// - After: `[a+b]`
    Add,

    /// Subtracts the top value from the second value on the stack.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: `[b, a]` TOP
    /// - After: `[b-a]`
    Subtract,

    /// Multiplies the top two values on the stack.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: `[b, a]` TOP
    /// - After: `[a*b]`
    Multiply,

    /// Divides the second value by the top value on the stack.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: `[b, a]` TOP
    /// - After: `[b/a]`
    Divide,

    /// Compares the top two values for equality.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: `[b, a]` TOP
    /// - After: `[a==b]`
    Equal,

    /// Compares the top two values for inequality.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: `[b, a]` TOP
    /// - After: `[a!=b]`
    NotEqual,

    /// Checks if the second value is less than the top value.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: `[b, a]` TOP
    /// - After: `[b < a]`
    LessThan,

    /// Checks if the second value is less than or equal to the top value.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: `[b, a]` TOP
    /// - After: `[b <= a]`
    LessEqual,

    /// Checks if the second value is greater than the top value.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: `[b, a]` TOP
    /// - After: `[b > a]`
    GreaterThan,

    /// Checks if the second value is greater than or equal to the top value.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: `[b, a]` TOP
    /// - After: `[b>=a]`
    GreaterEqual,

    /// Pops and prints the top value from the stack.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: `[value]`
    /// - After: `[]`
    Print,

    /// Removes the top value from the stack.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: `[value]`
    /// - After: `[]`
    Pop,

    /// Defines a new global variable and initializes it to the top value
    /// on the stack.
    ///
    /// ### Operand
    /// - 1 byte: index into constant pool for variable name
    /// - 3 bytes: index into constant pool for variable name (index > 255)
    ///
    /// ### Stack effect
    /// - Before: `[value]`
    /// - After: `[]`
    DefineGlobal,
    /// The long version of [`OpCode::DefineGlobal`]
    DefineGlobalLong,

    /// Pushes the value of a global variable onto the stack.
    ///
    /// ### Operand
    /// - 1 byte: index into constant pool for variable name
    /// - 3 bytes: index into constant pool for variable name (index > 255)
    ///
    /// ### Stack effect
    /// - Before: `[]`
    /// - After: `[value]`
    GetGlobal,
    /// The long version of [`OpCode::GetGlobal`]
    GetGlobalLong,

    /// Sets the global variable to the top value of the stack.
    ///
    /// ### Operand
    /// - 1 byte: index into constant pool for variable name
    /// - 3 bytes: index into constant pool for variable name (index > 255)
    ///
    /// ### Stack effect
    /// - Before: `[value]`
    /// - After: `[value]`
    SetGlobal,
    /// The long version of [`OpCode::SetGlobal`]
    SetGlobalLong,

    /// Pushes the value of a local variable onto the stack.
    ///
    /// ### Operand
    /// - 1 byte: index into stack for variable name
    /// - 3 bytes: index into stack for variable name (index > 255)
    ///
    /// ### Stack effect
    /// - Before: `[]`
    /// - After: `[value]`
    GetLocal,
    /// The long version of [`OpCode::GetLocal`]
    GetLocalLong,

    /// Sets the local variable to the top value of the stack.
    ///
    /// ### Operand
    /// - 1 byte: index into stack for variable name
    /// - 3 bytes: index into constant pool for variable name (index > 255)
    ///
    /// ### Stack effect
    /// - Before: `[value]`
    /// - After: `[value]`
    SetLocal,
    /// Long version of  [`OpCode::SetLocal`]
    SetLocalLong,

    /// Jump a # of bytes.
    ///
    /// ### Operand
    /// - 2 bytes: the number of bytes to jump
    ///
    /// ### Stack effect
    /// - Before: `[value]`
    /// - After: `[value]`
    Jump,

    /// Jump a # of bytes if the top value of the stack is false.
    ///
    /// ### Operand
    /// - 2 bytes: the number of bytes to jump
    ///
    /// ### Stack effect
    /// - Before: `[value]`
    /// - After: `[value]`
    JumpIfFalse,

    /// Jump a # of bytes backwards.
    ///
    /// ### Operand
    /// - 2 bytes: the number of bytes to jump
    ///
    /// ### Stack effect
    /// - Before: `[value]`
    /// - After: `[value]`
    Loop,

    /// Calls the function at the n'th position from the top
    /// of the stack..
    ///
    /// ### Operand
    /// - 1 byte: the number of arguments this function has
    ///
    /// ### Stack effect
    /// - Before: `[value]`
    /// - After: `[value]`
    Call,

    /// Exits the function and returns the value on the top of the stack
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: `[value]`
    /// - After: `[]`
    Return,

    /// Crates a closure from a function and stuffs it into the heap
    ///
    /// ### Operand
    /// - 1 byte: index into the heap of where function is located
    /// - 3 bytes: index into the heap of where function is located
    ///
    /// ### Stack effect
    /// - Before: `[]`
    /// - After: `[value]`
    Closure,
    /// Long version of  [`OpCode::Closure`]
    ClosureLong,

    /// No operation, discards the byte.
    Nop,
}

impl OpCode {
    pub fn to_long(self) -> Self {
        match self {
            OpCode::LoadConstant => OpCode::LoadConstantLong,
            OpCode::DefineGlobal => OpCode::DefineGlobalLong,
            OpCode::GetGlobal => OpCode::GetGlobalLong,
            OpCode::GetLocal => OpCode::GetLocalLong,
            OpCode::SetLocal => OpCode::SetLocalLong,
            OpCode::Closure => OpCode::ClosureLong,
            _ => self,
        }
    }
}
