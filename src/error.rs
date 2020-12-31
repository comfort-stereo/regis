use std::fs;
use std::str::from_utf8;

use crate::ast::location::{Location, Position};
use crate::interpreter::ValueType;

#[derive(Debug)]
pub struct RegisError {
    pub location: Option<Location>,
    pub variant: RegisErrorVariant,
}

impl RegisError {
    pub fn new(location: Option<Location>, variant: RegisErrorVariant) -> Self {
        Self { location, variant }
    }
}

#[derive(Debug)]
pub enum RegisErrorVariant {
    UndefinedBinaryOperation {
        operation: String,
        target_type: ValueType,
        other_type: ValueType,
    },
    UndefinedUnaryOperation {
        operation: String,
        target_type: ValueType,
    },
    IndexOutOfBoundsError {
        message: String,
    },
    ArgumentCountError {
        function_name: Option<String>,
        required: usize,
        actual: usize,
    },
    TypeError {
        message: String,
    },
    ModuleDoesNotExistError {
        path: String,
    },
    ParseError {
        expected: Vec<String>,
    },
}

#[derive(Debug)]
struct RegisErrorMessageDisplay {
    label: String,
    message: String,
}

impl RegisError {
    pub fn show(&self) -> String {
        let message = self.display_message();
        let mut output = Vec::new();

        if let Some(location) = &self.location {
            let Location { path, .. } = location.clone();
            let Position { line, column, .. } = location.start;
            if let Some(path) = path {
                if let Ok(source) = fs::read_to_string(&path) {
                    let code = Self::show_location(&location, &source);
                    let padding = " ".repeat(line.to_string().len());

                    output.push(format!("- error -> {} -> {}:{}", path, line, column));
                    output.push(format!("{} |", padding));
                    output.push(format!("{} | {}", line, code));
                    output.push(format!("{} |{}^", padding, " ".repeat(column)));
                }
            } else {
                output.push(format!("- error -> {}:{}", line, column));
            }
        }

        output.push(format!("- error -> {}", message));
        output.join("\n")
    }

    fn display_message(&self) -> String {
        match &self.variant {
            RegisErrorVariant::UndefinedBinaryOperation {
                operation,
                target_type,
                other_type,
            } => {
                format!(
                    "Operation '{}' is not defined for types '{}' and '{}'.",
                    operation, target_type, other_type,
                )
            }
            RegisErrorVariant::UndefinedUnaryOperation {
                operation,
                target_type,
            } => {
                format!(
                    "Operation '{}' is not defined for type '{}'.",
                    operation, target_type
                )
            }
            RegisErrorVariant::IndexOutOfBoundsError { message } => message.into(),
            RegisErrorVariant::ArgumentCountError {
                function_name,
                required,
                actual,
            } => match function_name {
                Some(function_name) => format!(
                    "Attempted to call function '{}()' with {} arguments. It requires at least {}.",
                    function_name, actual, required
                ),
                None => format!(
                    "Attempted to call anonymous function with {} arguments. It requires at least {}.",
                    actual, required
                ),
            },
            RegisErrorVariant::TypeError { message } => message.into(),
            RegisErrorVariant::ModuleDoesNotExistError { path } => format!(
                "Imported module at path '{}' does not exist.",
                path,
            ),
            RegisErrorVariant::ParseError { expected } => format!(
                "Invalid syntax, expected: {}",
                expected.join(" | "),
            )
        }
    }

    fn show_location(location: &Location, source: &str) -> String {
        let bytes = source.as_bytes();

        let mut start = location.start.index.min(bytes.len() - 1).max(0);
        let mut end = start;

        while start > 0 && (bytes[start] as char) != '\n' {
            start -= 1;
        }
        while end < source.len() && (bytes[end] as char) != '\n' {
            end += 1;
        }

        from_utf8(&bytes[start..end]).unwrap().trim().into()
    }
}
