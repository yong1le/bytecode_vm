use std::fmt;

use crate::ast::expr::Expr;

use super::literal::Literal;

/// Errors that can occur during scanning/lexical analysis.
#[derive(Debug, Clone)]
pub enum ScanError {
    UnterminatedString(u32),
    UnexpectedCharacter(u32, char),
}

/// Syntactical errors that can occur during parsing.
#[derive(Debug, Clone)]
pub enum SyntaxError {
    ScanError(ScanError),
    ExpectedChar(u32, String, String),
    ExpectedExpression(u32, String),
    UnexpectedEOF,
    InvalidAssignment(u32, Expr),
}

#[derive(Debug, Clone)]
pub enum SemanticError {
    UndeclaredLocalInInitializer(u32),
    AlreadyDeclared(u32, String),
    TopReturn(u32),
    TopThis(u32),
    ReturnValueInInit(u32),
    SelfInheritance(u32, String),
    TopSuper(u32),
    TopClassSuper(u32),
}

/// Runtime errors that occur while executing the program.
#[derive(Debug, Clone)]
pub enum RuntimeError {
    NameError(u32, String),
    UnaryOperandMismatch(u32),
    BinaryOperandMismatch(u32),
    UnimplementedOperand(u32, String),
    InvalidCall(u32, String),
    FunctionCallArityMismatch(u32, usize, usize),
    InvalidPropertyAccess(u32, String, String),
    InheritFromNonClass(u32, String, String),
    ReturnValue(Literal), // Used to return value from functions
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnterminatedString(line) => {
                write!(f, "[line {}] Error: Unterminated string.", line)
            }
            Self::UnexpectedCharacter(line, ch) => {
                write!(f, "[line {}] Error: Unexpected character: {}", line, ch)
            }
        }
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ScanError(s) => {
                write!(f, "{s}")
            }
            Self::ExpectedChar(line, actual, expected) => {
                write!(
                    f,
                    "[line {}] Error at '{}': Expected {}.",
                    line, actual, expected
                )
            }
            Self::ExpectedExpression(line, actual) => {
                write!(
                    f,
                    "[line {}] Error at '{}': Expected expression.",
                    line, actual
                )
            }
            Self::UnexpectedEOF => {
                write!(f, "Error: Unexpected End of File.")
            }
            Self::InvalidAssignment(line, assignee) => {
                write!(
                    f,
                    "[line {}] Error: Attempting to assign to {}.",
                    line,
                    match assignee {
                        Expr::Binary(_, _, _) | Expr::And(_, _) | Expr::Or(_, _) =>
                            "a binary operation",
                        Expr::Grouping(_) => "parentheses",
                        Expr::Unary(_, _) => "a unary operation",
                        Expr::Literal(_) => "a literal value",
                        Expr::Variable(_) => "a variable",
                        Expr::Assign(_, _) => "a assignment",
                        Expr::Call(_, _, _) => "a function call",
                        Expr::Get(_, _) => "a property",
                        Expr::Set(_, _, _) => "a property",
                        Expr::This(_) => "a variable",
                        Expr::Super(_, _) => "a property",
                    }
                )
            }
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::NameError(line, s) => {
                write!(f, "[line {}] Error: '{}' is not defined.", line, s)
            }
            RuntimeError::UnaryOperandMismatch(line) => {
                write!(f, "[line {}] Error: Operand must be a number.", line)
            }
            RuntimeError::BinaryOperandMismatch(line) => {
                write!(f, "[line {}] Error: Both operands must be numbers.", line)
            }
            RuntimeError::UnimplementedOperand(line, op) => write!(
                f,
                "[line {}] Error at '{}': Operand unimplemented.",
                op, line
            ),
            RuntimeError::InvalidCall(line, callable) => {
                write!(f, "[line {}] Error: '{}' is not callable.", line, callable)
            }
            RuntimeError::FunctionCallArityMismatch(line, expected, actual) => write!(
                f,
                "[line {}] Error: Expected {} arguments but received {}.",
                line, expected, actual
            ),
            RuntimeError::InvalidPropertyAccess(line, id, prop) => write!(
                f,
                "[line {}] Error: Cannot access '{}' on non-instance value '{}'.",
                line, prop, id
            ),
            RuntimeError::InheritFromNonClass(line, class, parent) => write!(
                f,
                "[line {}] Error: '{}' attempting to inherit from non-class '{}'",
                line, class, parent
            ),
            RuntimeError::ReturnValue(l) => write!(f, "Returning {}", l),
        }
    }
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SemanticError::UndeclaredLocalInInitializer(line) => write!(
                f,
                "[line {}] Error: Can't read local variable in its own initializer.",
                line
            ),
            SemanticError::AlreadyDeclared(line, id) => write!(
                f,
                "[line {}] Error: Already a variable '{}' in this scope.",
                line, id
            ),
            SemanticError::TopReturn(line) => write!(
                f,
                "[line {}] Error at 'return': Can't return from top-level code.",
                line
            ),
            SemanticError::TopThis(line) => write!(
                f,
                "[line {}] Error at 'this': Can't use this outside of class methods.",
                line
            ),
            SemanticError::ReturnValueInInit(line) => write!(
                f,
                "[line {}] Error at 'return': Cannot return value in a class constructor",
                line
            ),
            SemanticError::SelfInheritance(line, id) => write!(
                f,
                "[line {}] Error '{}': A class cannot inherit from itself.",
                line, id
            ),
            SemanticError::TopSuper(line) => write!(
                f,
                "[line {}] Error at 'super': Can't 'super' in top-level code.",
                line
            ),
            SemanticError::TopClassSuper(line) => write!(
                f,
                "[line {}] Error at 'super': Can't use 'super' in class without a parent.",
                line
            ),
        }
    }
}
