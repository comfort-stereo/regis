use crate::parser::ParseRule;
use crate::value_type::ValueType;
use pest::error::Error as ParseError;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum InterpreterError {
    ParseError {
        error: ParseError<ParseRule>,
    },
    UndefinedVariableAccess {
        name: String,
    },
    UndefinedVariableAssignment {
        name: String,
    },
    VariableRedeclaration {
        name: String,
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

impl Display for InterpreterError {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        match self {
            Self::ParseError { error } => {
                write!(formatter, "{}", error)
            }
            Self::UndefinedVariableAccess { name } => {
                write!(
                    formatter,
                    "Attempted to access undefined variable '{}'",
                    name
                )
            }
            Self::UndefinedVariableAssignment { name } => {
                write!(
                    formatter,
                    "Attempted assignment to undefined variable '{}'",
                    name
                )
            }
            Self::VariableRedeclaration { name } => {
                write!(
                    formatter,
                    "Redeclaration of previously defined variable '{}'",
                    name
                )
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
