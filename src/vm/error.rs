use pest::error::Error as ParseError;
use std::fmt::{Display, Formatter, Result};

use crate::ast::parser::ParseRule;
use crate::vm;

use vm::value::ValueType;

#[derive(Debug)]
pub enum VmError {
    ParseError {
        error: ParseError<ParseRule>,
    },
    UndefinedBinaryOperation {
        operation: String,
        target_type: ValueType,
        other_type: ValueType,
    },
    UndefinedUnaryOperation {
        operation: String,
        target_type: ValueType,
    },
    InvalidIndexAccess {
        target_type: ValueType,
        index: String,
    },
    InvalidIndexAssignment {
        target_type: ValueType,
        index: String,
    },
}

impl Display for VmError {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        match self {
            Self::ParseError { error } => {
                write!(formatter, "{}", error)
            }
            Self::UndefinedBinaryOperation {
                operation,
                target_type,
                other_type,
            } => {
                write!(
                    formatter,
                    "Operation '{}' is not defined for types '{}' and '{}'",
                    operation, target_type, other_type,
                )
            }
            Self::UndefinedUnaryOperation {
                operation,
                target_type,
            } => {
                write!(
                    formatter,
                    "Operation '{}' is not defined for type '{}'",
                    operation, target_type
                )
            }
            Self::InvalidIndexAccess { target_type, index } => {
                write!(
                    formatter,
                    "Attempted to get invalid index '{}' of type '{}'",
                    index, target_type
                )
            }
            Self::InvalidIndexAssignment { target_type, index } => {
                write!(
                    formatter,
                    "Attempted to set invalid index '{}' of type '{}'",
                    index, target_type,
                )
            }
        }
    }
}
