use crate::value_type::ValueType;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum VmError {
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
            VmError::UndefinedVariableAccess { name } => {
                write!(
                    formatter,
                    "Attempted to access undefined variable '{}'",
                    name
                )
            }
            VmError::UndefinedVariableAssignment { name } => {
                write!(
                    formatter,
                    "Attempted assignment to undefined variable '{}'",
                    name
                )
            }
            VmError::VariableRedeclaration { name } => {
                write!(
                    formatter,
                    "Redeclaration of previously defined variable '{}'",
                    name
                )
            }
            VmError::UndefinedBinaryOperation {
                operation,
                target_type,
                other_type,
            } => {
                write!(
                    formatter,
                    "Operation {} is not defined for types '{}' and '{}'",
                    operation, target_type, other_type,
                )
            }
            VmError::InvalidIndexAccess { target_type, index } => {
                write!(
                    formatter,
                    "Attempted to get invalid index '{}' of type '{}'",
                    index, target_type
                )
            }
            VmError::InvalidIndexAssignment { target_type, index } => {
                write!(
                    formatter,
                    "Attempted to set invalid index '{}' of type '{}'",
                    index, target_type,
                )
            }
        }
    }
}
