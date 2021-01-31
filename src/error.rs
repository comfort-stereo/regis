use std::str::from_utf8;

use crate::interpreter::ValueType;
use crate::source::{Location, Span};

#[derive(Debug)]
pub struct RegisError {
    location: Option<Location>,
    variant: RegisErrorVariant,
}

impl RegisError {
    pub fn new(location: Option<Location>, variant: RegisErrorVariant) -> Self {
        Self { location, variant }
    }

    pub fn location(&self) -> &Option<Location> {
        &self.location
    }

    pub fn variant(&self) -> &RegisErrorVariant {
        &self.variant
    }
}

#[derive(Debug)]
pub enum RegisErrorVariant {
    UndefinedUnaryOperation {
        operation: String,
        right_type: ValueType,
    },
    UndefinedBinaryOperation {
        operation: String,
        left_type: ValueType,
        right_type: ValueType,
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
        message: String,
    },
}

#[derive(Debug)]
struct RegisErrorMessageDisplay {
    label: String,
    message: String,
}

impl RegisError {
    pub fn show(&self, source: Option<&str>) -> String {
        let message = self.display_message();
        let mut output = Vec::new();

        if let Some(source) = source {
            if let Some(location) = &self.location() {
                let (line, column, code) = Self::span_info(location.span(), &source);

                if let Some(path) = &location.path() {
                    output.push(format!("- error -> {} -> {}:{}", path, line, column));
                } else {
                    output.push(format!("- error -> {}:{}", line, column));
                }

                let padding = " ".repeat(line.to_string().len());
                output.push(format!("{} |", padding));
                output.push(format!("{} | {}", line, code));
                output.push(format!("{} |{}^", padding, " ".repeat(column)));
            }
        }

        output.push(format!("- error -> {}", message));
        output.join("\n")
    }

    fn display_message(&self) -> String {
        match &self.variant {
            RegisErrorVariant::UndefinedBinaryOperation {
                operation,
                left_type: target_type,
                right_type: other_type,
            } => {
                format!(
                    "Operation '{}' is not defined for types '{}' and '{}'.",
                    operation, target_type, other_type,
                )
            }
            RegisErrorVariant::UndefinedUnaryOperation {
                operation,
                right_type: target_type,
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
            RegisErrorVariant::ParseError { message } => format!( "Invalid syntax. {}", message),
        }
    }

    fn span_info(span: &Span, source: &str) -> (usize, usize, String) {
        fn is_newline(string: &str, index: usize) -> bool {
            string.is_char_boundary(index) && string.as_bytes()[index] as char == '\n'
        }

        let bytes = source.as_bytes();
        let code = {
            let mut start = span.start().min(bytes.len() - 1).max(0);
            let mut end = start;

            while start > 0 && !is_newline(source, start) {
                start -= 1;
            }

            while end < source.len() && !is_newline(source, end) {
                end += 1;
            }

            from_utf8(&bytes[start..end]).unwrap()
        };

        let (line, column) = {
            let mut line = 1;
            let mut column = 1;

            for (i, character) in source.char_indices() {
                if i == span.start() {
                    break;
                }

                if character == '\n' {
                    line += 1;
                    column = 1;
                } else {
                    column += 1;
                }
            }

            (line, column)
        };

        (line, column, code.into())
    }
}
