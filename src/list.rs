use crate::shared::Shared;
use crate::value::Value;
use crate::value_type::ValueType;
use crate::vm_error::VmError;

#[derive(Debug)]
pub struct List {
    values: Vec<Value>,
}

impl List {
    fn new() -> List {
        List { values: Vec::new() }
    }

    pub fn create() -> Shared<List> {
        Shared::new(List::new())
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
                        index: index.to_string(),
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

    pub fn concat(&self, other: Shared<List>) -> Shared<List> {
        let mut result = Self::new();
        result.reserve(self.count() + other.borrow().count());

        for value in &self.values {
            result.push(value.clone())
        }
        for value in &other.borrow().values {
            result.push(value.clone())
        }

        Shared::new(result)
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
