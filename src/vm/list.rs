use std::hash::{Hash, Hasher};

use crate::shared::SharedMutable;
use crate::vm::error::VmError;

use super::rid::rid;
use super::value::{Value, ValueType};

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

    pub fn to_string(&self) -> String {
        format!(
            "[{}]",
            self.inner
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    pub fn get(&self, index: Value) -> Result<Value, VmError> {
        match index {
            Value::Number(number) => {
                let index = number as usize;
                if number < 0f64 || index >= self.inner.len() {
                    return Ok(Value::Null);
                }

                Ok(self.inner[index].clone())
            }
            _ => Err(VmError::InvalidIndexAccess {
                target_type: self.type_of(),
                index: index.to_string(),
            }),
        }
    }

    pub fn set(&mut self, index: Value, value: Value) -> Result<(), VmError> {
        match index {
            Value::Number(number) => {
                let index = number as usize;
                if number < 0f64 || index >= self.inner.len() {
                    return Err(VmError::InvalidIndexAssignment {
                        target_type: self.type_of(),
                        index: number.to_string(),
                    });
                }

                self.inner[index] = value;
                Ok(())
            }
            _ => Err(VmError::InvalidIndexAssignment {
                target_type: self.type_of(),
                index: index.to_string(),
            }),
        }
    }

    pub fn reserve(&mut self, capacity: usize) {
        self.inner.reserve(capacity);
    }

    pub fn concat(&self, other: &SharedMutable<Self>) -> SharedMutable<Self> {
        let mut result = Self::new();
        result.reserve(self.len() + other.borrow().len());

        for value in &self.inner {
            result.push(value.clone())
        }
        for value in &other.borrow().inner {
            result.push(value.clone())
        }

        result.into()
    }

    pub fn push(&mut self, value: Value) {
        self.inner.push(value)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}
