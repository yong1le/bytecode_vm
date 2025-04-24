use std::fmt;

use crate::ast::expr::Expr;

#[derive(Debug, Clone)]
pub enum ScanError {
    UnterminatedString(u32),
    UnexpectedCharacter(u32, char),
}

#[derive(Debug, Clone)]
pub enum ParseError {
    ScanError(ScanError),
    ExpectedChar(u32, String, String),
    ExpectedExpression(u32, String),
    UnexpectedEOF,
    InvalidAssignment(u32, Expr),
}

#[derive(Debug, Clone)]
pub enum EvalError {
    TypeError(u32, &'static str),
    NameError(u32, String),
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

impl fmt::Display for ParseError {
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
                    line, match assignee {
                        Expr::Binary(_,_,_) => "a binary operation",
                        Expr::Grouping(_) => "parentheses",
                        Expr::Unary(_,_) => "a unary operation",
                        Expr::Literal(_)=> "a literal value",
                        Expr::Variable(_) => "a variable",
                        Expr::Assign(_, _) => "a assignment"
                    }
                )
            }
        }
    }
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::TypeError(line, s) => write!(f, "[line {}] Error: {}", line, s),
            EvalError::NameError(line, s) => {
                write!(f, "[line {}] Error: Variable '{}' is not defined.", line, s)
            }
        }
    }
}
