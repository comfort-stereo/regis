use std::str::from_utf8;

use crate::ast::error::ParseError;
use crate::ast::location::{Location, Position};
use crate::vm::{VmError, VmErrorVariant};

#[derive(Debug)]
pub enum InterpreterError {
    ParseError(ParseError),
    VmError(VmError),
}

#[derive(Debug)]
struct InterpreterErrorDisplay {
    label: String,
    message: String,
    location: Option<Location>,
}

impl InterpreterError {
    pub fn show(&self, source: &str) -> String {
        let InterpreterErrorDisplay {
            label,
            message,
            location,
        } = match self {
            InterpreterError::ParseError(error) => self.display_parse_error(error),
            InterpreterError::VmError(error) => self.display_vm_error(error),
        };

        let mut output = Vec::new();
        output.push(format!("-> ERROR({})", label));
        output.push(format!("-> {}", message));

        if let Some(location) = location {
            let Location { path, .. } = location.clone();
            let Position { line, column, .. } = location.start;
            let code = Self::display_location(&location, source);
            let padding = " ".repeat(line.to_string().len());

            output.push(match path {
                Some(path) => {
                    format!("-> {}:{}:{}", path, line, column)
                }
                None => {
                    format!("-> {}:{}", line, column)
                }
            });

            output.push(format!("{} |", padding));
            output.push(format!("{} | {}", line, code));
            output.push(format!("{} |{}^", padding, " ".repeat(column)));
        }

        output.join("\n")
    }

    fn display_parse_error(
        &self,
        ParseError { location, expected }: &ParseError,
    ) -> InterpreterErrorDisplay {
        let message = format!(
            "Invalid syntax, expected: [{}]",
            expected
                .iter()
                .map(|token| format!("\"{}\"", token))
                .collect::<Vec<_>>()
                .join(" | ")
        );

        InterpreterErrorDisplay {
            label: "SyntaxError".into(),
            message,
            location: Some(location.clone()),
        }
    }

    fn display_vm_error(&self, error: &VmError) -> InterpreterErrorDisplay {
        let message = match &error.variant {
            VmErrorVariant::UndefinedBinaryOperation {
                operation,
                target_type,
                other_type,
            } => {
                format!(
                    "Operation '{}' is not defined for types '{}' and '{}'.",
                    operation, target_type, other_type,
                )
            }
            VmErrorVariant::UndefinedUnaryOperation {
                operation,
                target_type,
            } => {
                format!(
                    "Operation '{}' is not defined for type '{}'.",
                    operation, target_type
                )
            }
            VmErrorVariant::InvalidIndexAccess { target_type, index } => {
                format!(
                    "Attempted to get invalid index '{}' of type '{}'.",
                    index, target_type
                )
            }
            VmErrorVariant::InvalidIndexAssignment { target_type, index } => {
                format!(
                    "Attempted to set invalid index '{}' of type '{}'.",
                    index, target_type,
                )
            }
        };

        InterpreterErrorDisplay {
            label: "VmError".into(),
            message,
            location: None,
        }
    }

    fn display_location(location: &Location, source: &str) -> String {
        let bytes = source.as_bytes();

        let mut start = location.start.index;
        let mut end = location.start.index;

        while start > 0 && (bytes[start] as char) != '\n' {
            start -= 1;
        }
        while end < source.len() && (bytes[end] as char) != '\n' {
            end += 1;
        }

        from_utf8(&bytes[start..end]).unwrap().trim().into()
    }
}
