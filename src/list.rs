use crate::shared::SharedMutable;
use crate::value::Value;
use crate::value_type::ValueType;
use crate::vm_error::VmError;

#[derive(Debug)]
pub struct List {
    values: Vec<Value>,
}

impl PartialEq for List {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for List {}

impl List {
    pub fn new() -> Self {
        List { values: Vec::new() }
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
            self.values()
                .into_iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    pub fn get(&self, index: Value) -> Result<Value, VmError> {
        match index {
            Value::Number(number) => {
                let index = number as usize;
                if number < 0f64 || index >= self.values.len() {
                    return Ok(Value::Null);
                }

                Ok(self.values[index].clone())
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
                if number < 0f64 || index >= self.values.len() {
                    return Err(VmError::InvalidIndexAssignment {
                        target_type: self.type_of(),
                        index: number.to_string(),
                    });
                }

                self.values[index] = value;
                Ok(())
            }
            _ => Err(VmError::InvalidIndexAssignment {
                target_type: self.type_of(),
                index: index.to_string(),
            }),
        }
    }

    pub fn reserve(&mut self, capacity: usize) {
        self.values.reserve(capacity);
    }

    pub fn concat(&self, other: SharedMutable<Self>) -> SharedMutable<Self> {
        let mut result = Self::new();
        result.reserve(self.count() + other.borrow().count());

        for value in &self.values {
            result.push(value.clone())
        }
        for value in &other.borrow().values {
            result.push(value.clone())
        }

        SharedMutable::new(result)
    }

    pub fn push(&mut self, value: Value) {
        self.values.push(value)
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }

    pub fn values(&self) -> &Vec<Value> {
        &self.values
    }
}
