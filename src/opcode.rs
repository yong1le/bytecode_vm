use derive_more::TryFrom;

#[derive(Debug, TryFrom, Clone, Copy)]
#[try_from(repr)]
#[repr(u8)]
pub enum OpCode {
    /// Loads a constant from the constant pool onto the stack.
    ///
    /// ### Operand
    /// - normal: 1 byte: index into the constant pool
    /// - long: 3 bytes: index into the constant pool (for large constant pools)
    ///
    /// ### Stack effect
    /// - Before: []
    /// - After: [value]
    Constant,
    ConstantLong,

    /// Negates the value on top of the stack.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: [value]
    /// - After: [-value]
    Negate,

    /// Applies logical NOT to the value on top of the stack.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: [value]
    /// - After: [!value]
    Not,

    /// Adds the top two values on the stack.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: [b, a] TOP
    /// - After: [a+b]
    Add,

    /// Subtracts the top value from the second value on the stack.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: [b, a] TOP
    /// - After: [b-a]
    Subtract,

    /// Multiplies the top two values on the stack.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: [b, a] TOP
    /// - After: [a*b]
    Multiply,

    /// Divides the second value by the top value on the stack.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: [b, a] TOP
    /// - After: [b/a]
    Divide,

    /// Compares the top two values for equality.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: [b, a] TOP
    /// - After: [a==b]
    Equal,

    /// Compares the top two values for inequality.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: [b, a] TOP
    /// - After: [a!=b]
    NotEqual,

    /// Checks if the second value is less than the top value.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: [b, a] TOP
    /// - After: [b < a]
    LessThan,

    /// Checks if the second value is less than or equal to the top value.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: [b, a] TOP
    /// - After: [b <= a]
    LessEqual,

    /// Checks if the second value is greater than the top value.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: [b, a] TOP
    /// - After: [b > a]
    GreaterThan,

    /// Checks if the second value is greater than or equal to the top value.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: [b, a] TOP
    /// - After: [b>=a]
    GreaterEqual,

    /// Pops and prints the top value from the stack.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: [value]
    /// - After: []
    Print,

    /// Removes the top value from the stack.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: [value]
    /// - After: []
    Pop,

    /// Defines a new global variable and initializes it to the top value
    /// on the stack.
    ///
    /// ### Operand
    /// - 1 byte: index into constant pool for variable name
    ///
    /// ### Stack effect
    /// - Before: [value]
    /// - After: []
    DefineGlobal,
    DefineGlobalLong,

    /// Pushes the value of a global variable onto the stack.
    ///
    /// ### Operand
    /// - 1 byte: index into constant pool for variable name
    ///
    /// ### Stack effect
    /// - Before: []
    /// - After: [value]
    GetGlobal,
    GetGlobalLong,

    /// Sets the global variable and to the top value of the stack.
    ///
    /// ### Operand
    /// - 1 byte: index into constant pool for variable name
    ///
    /// ### Stack effect
    /// - Before: [value]
    /// - After: [value]
    SetGlobal,
    SetGlobalLong,

    /// Returns from the current function.
    ///
    /// ### Operand
    /// - None
    ///
    /// ### Stack effect
    /// - Before: []
    /// - After: []
    Return,
}

impl OpCode {
    pub fn to_long(self) -> Self {
        match self {
            OpCode::Constant => OpCode::ConstantLong,
            OpCode::DefineGlobal => OpCode::DefineGlobalLong,
            OpCode::GetGlobal => OpCode::GetGlobalLong,
            _ => self,
        }
    }
}
