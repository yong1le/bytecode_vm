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
}

/// Runtime errors that occur while executing the program.
#[derive(Debug, Clone)]
pub enum RuntimeError {
    TypeError(u32, String),
    NameError(u32, String),
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
                    }
                )
            }
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::TypeError(line, s) => write!(f, "[line {}] Error: {}", line, s),
            RuntimeError::NameError(line, s) => {
                write!(f, "[line {}] Error: Variable '{}' is not defined.", line, s)
            }
            RuntimeError::ReturnValue(literal) => write!(f, "Returning {}", literal),
        }
    }
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UndeclaredLocalInInitializer(line) => write!(
                f,
                "[line {}] Error: Can't read local variable in its own initializer.",
                line
            ),
            Self::AlreadyDeclared(line, id) => write!(
                f,
                "[line {}] Error: Already a variable '{}' in this scope.",
                line, id
            ),
            Self::TopReturn(line) => write!(
                f,
                "[line {}] Error at 'return': Can't return from top-level code.",
                line
            ),
        }
    }
}
