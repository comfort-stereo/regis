use crate::object::Object;
use crate::shared::Shared;
use crate::value::Value;

#[derive(Debug)]
pub struct List {
    values: Vec<Value>,
}

impl Object for List {
    fn to_boolean(&self) -> bool {
        true
    }

    fn to_string(&self) -> String {
        format!(
            "[{}]",
            self.values()
                .into_iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl List {
    fn new() -> List {
        List { values: Vec::new() }
    }

    pub fn create() -> Shared<List> {
        Shared::new(List::new())
    }

    pub fn get(&self, index: Value) -> Value {
        match index {
            Value::Number(number) => {
                let index = number as usize;
                if number <= 0f64 || index >= self.values.len() {
                    return Value::Null;
                }

                self.values[index].clone()
            }
            _ => {
                panic!("Lists cannot be indexed by type: '{}'", index.type_name());
            }
        }
    }

    pub fn set(&mut self, index: Value, value: Value) {
        match index {
            Value::Number(number) => {
                let index = number as usize;
                if number <= 0f64 || index >= self.values.len() {
                    panic!("Invalid assignment to index: {}", index);
                }

                self.values[index] = value;
            }
            _ => {
                panic!("Lists cannot be indexed by type: '{}'", index.type_name());
            }
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
