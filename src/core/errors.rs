use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum InterpretError {
    #[error("{0}")]
    Scan(ScanError),
    #[error("{0}")]
    Syntax(SyntaxError),
    #[error("{0}")]
    Compile(CompileError),
    #[error("{0}")]
    Runtime(RuntimeError),
    #[error("PANIC: {0}")]
    Panic(PanicError),
}

#[derive(Debug, Error, Clone)]
pub enum ScanError {
    #[error("[line {0}: Error: Unterminated string.")]
    UnterminatedString(u32),
    #[error("[line {0}: Error at '{1}': Unexpected character.")]
    UnexpectedCharacter(u32, char),
}

#[derive(Debug, Error, Clone)]
pub enum SyntaxError {
    #[error("[line {0}]: Error at '{1}': Expected {2}.")]
    ExpectedChar(u32, String, String),
    #[error("[line {0}]: Error at '{1}': Expected expression.")]
    ExpectedExpression(u32, String),
    #[error("Unexpected end of file.")]
    UnexpectedEOF,
    #[error("[line {0}]: Error at '=': Invalid assignment.")]
    InvalidAssignment(u32),
}

#[derive(Debug, Error, Clone)]
pub enum CompileError {
    #[error("[line {0}]: Invalid Operation Code: {1}")]
    InvalidOpCode(u32, u8),
    #[error("[line {0}]: Error': Cannot use variable in its own initializer.")]
    SelfInitialization(u32),
}

#[derive(Debug, Error, Clone)]
pub enum RuntimeError {
    #[error("[line {0}]: Error: '{1}' is not defined.")]
    NameError(u32, String),
    #[error("[line {0}]: Error: Operand(s) must be {1}.")]
    OperandMismatch(u32, String),
    #[error("[line {0}]: Error at '{1}': Object is not a callable.")]
    InvalidCall(u32, String),
    #[error("[line {0}]: Error: Expected {1} arguments, but received {2}.")]
    FunctionCallArityMismatch(u32, usize, usize),
    #[error("[line {0}]: Error: Cannot access '{1}' on non-instance value '{2}'.")]
    InvalidPropertyAccess(u32, String, String),
    #[error("[line {0}] Error: '{1}' attempting to inherit from non-class value '{2}'.")]
    InheritFromNonClass(u32, String, String),
}

#[derive(Debug, Error, Clone)]
pub enum PanicError {
    #[error("[line {0}]: Error: {1}")]
    General(u32, String),
    #[error("[line {0}]: Object pointer accessed after object was deallocated.")]
    DeallocatedObject(u32),
    #[error("[line {0}]: Passed non-object operand as variable.")]
    NonObjectVariable(u32),
}
