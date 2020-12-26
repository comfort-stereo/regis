use std::str::from_utf8;

use crate::ast::error::ParseError;
use crate::ast::location::Location;
use crate::vm::{VmError, VmErrorVariant};

#[derive(Debug)]
pub enum InterpreterError {
    ParseError(ParseError),
    VmError(VmError),
}

const PREVIOUS_LINE_COUNT: usize = 3;
const NEXT_LINE_COUNT: usize = 3;

impl InterpreterError {
    pub fn display(&self, source: &str) -> String {
        match self {
            InterpreterError::ParseError(error) => self.display_parse_error(error, source),
            InterpreterError::VmError(error) => self.display_vm_error(error, source),
        }
    }

    fn display_parse_error(&self, error: &ParseError, source: &str) -> String {
        let location = Self::display_location(&error.location, source);
        format!("ParseError: Code failed to parse.\n{}", location)
    }

    fn display_vm_error(&self, error: &VmError, source: &str) -> String {
        let message = match &error.variant {
            VmErrorVariant::UndefinedBinaryOperation {
                operation,
                target_type,
                other_type,
            } => {
                format!(
                    "Operation '{}' is not defined for types '{}' and '{}'",
                    operation, target_type, other_type,
                )
            }
            VmErrorVariant::UndefinedUnaryOperation {
                operation,
                target_type,
            } => {
                format!(
                    "Operation '{}' is not defined for type '{}'",
                    operation, target_type
                )
            }
            VmErrorVariant::InvalidIndexAccess { target_type, index } => {
                format!(
                    "Attempted to get invalid index '{}' of type '{}'",
                    index, target_type
                )
            }
            VmErrorVariant::InvalidIndexAssignment { target_type, index } => {
                format!(
                    "Attempted to set invalid index '{}' of type '{}'",
                    index, target_type,
                )
            }
        };

        let location = error.location.clone().map_or("".to_owned(), |location| {
            Self::display_location(&location, source)
        });

        format!("VmError: {}\n{}", message, location)
    }

    fn display_location(location: &Location, source: &str) -> String {
        let bytes = source.as_bytes();

        let mut start = location.start.index;
        let mut end = location.end.index;

        {
            let mut lines = 0;
            while start > 0 {
                if (bytes[start] as char) == '\n' {
                    lines += 1;
                }
                if lines == PREVIOUS_LINE_COUNT {
                    break;
                }

                start -= 1;
            }
        }

        {
            let mut lines = 0;
            while end > 0 {
                if (bytes[end] as char) == '\n' {
                    lines += 1;
                }
                if lines == NEXT_LINE_COUNT {
                    break;
                }

                end += 1;
            }
        }

        let code: String = from_utf8(&bytes[start..end]).unwrap().into();
        format!(
            "Line: {}, Col: {}:\n{}\n{}\n{}^",
            location.start.line,
            location.start.column,
            "*".repeat(80),
            code,
            "-".repeat(location.start.column - 1),
        )
    }
}
