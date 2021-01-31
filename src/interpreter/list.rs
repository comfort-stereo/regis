use std::fmt::{Display, Formatter, Result as FormatResult};
use std::hash::{Hash, Hasher};

use crate::error::RegisError;
use crate::shared::SharedMutable;

use super::rid::rid;
use super::value::{Value, ValueType};
use super::RegisErrorVariant;

#[derive(Debug)]
pub struct List {
    id: usize,
    inner: Vec<Value>,
}

impl PartialEq for List {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for List {}

impl Hash for List {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl Display for List {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        write!(
            formatter,
            "[{}]",
            self.inner
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl Default for List {
    fn default() -> Self {
        List::new()
    }
}

impl List {
    pub fn new() -> Self {
        Self {
            id: rid(),
            inner: Vec::new(),
        }
    }

    pub fn type_of(&self) -> ValueType {
        ValueType::List
    }

    pub fn to_boolean(&self) -> bool {
        true
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn get(&self, index: &Value) -> Result<Value, RegisError> {
        match index {
            Value::Int(int) => {
                let positive = *int as usize;
                if *int < 0 || positive >= self.inner.len() {
                    return Ok(Value::Null);
                }

                Ok(self.inner[positive].clone())
            }
            _ => Err(RegisError::new(
                None,
                RegisErrorVariant::TypeError {
                    message: format!(
                        "Lists cannot be indexed by type '{}', only '{}' is allowed.",
                        index.type_of(),
                        ValueType::Int
                    ),
                },
            )),
        }
    }

    pub fn set(&mut self, index: Value, value: Value) -> Result<(), RegisError> {
        match index {
            Value::Int(int) => {
                let index = int as usize;
                if int < 0 || index >= self.inner.len() {
                    return Err(RegisError::new(
                        None,
                        RegisErrorVariant::IndexOutOfBoundsError {
                            message: format!(
                                "Attempted to set invalid list index '{}'.",
                                value.to_string()
                            ),
                        },
                    ));
                }

                self.inner[index] = value;
                Ok(())
            }
            _ => Err(RegisError::new(
                None,
                RegisErrorVariant::TypeError {
                    message: format!(
                        "Lists cannot be indexed by type '{}', only '{}' is allowed.",
                        index.type_of(),
                        ValueType::Int
                    ),
                },
            )),
        }
    }

    pub fn reserve(&mut self, capacity: usize) {
        self.inner.reserve(capacity);
    }

    pub fn concat(&self, other: &Self) -> SharedMutable<Self> {
        let mut result = Self::new();
        result.reserve(self.len() + other.len());

        for value in &self.inner {
            result.push(value.clone())
        }
        for value in &other.inner {
            result.push(value.clone())
        }

        result.into()
    }

    pub fn push(&mut self, value: Value) {
        self.inner.push(value)
    }
}
