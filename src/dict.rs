use indexmap::IndexMap;
use std::hash::{Hash, Hasher};

use crate::oid::oid;
use crate::shared::SharedMutable;
use crate::value::Value;
use crate::value_type::ValueType;
#[derive(Debug)]
pub struct Dict {
    id: usize,
    inner: IndexMap<Value, Value>,
}

impl PartialEq for Dict {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Dict {}

impl Hash for Dict {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl Dict {
    pub fn new() -> Self {
        Self {
            id: oid(),
            inner: IndexMap::new(),
        }
    }

    pub fn type_of(&self) -> ValueType {
        ValueType::Dict
    }

    pub fn to_boolean(&self) -> bool {
        true
    }

    pub fn to_string(&self) -> String {
        format!(
            "{{{}}}",
            self.inner
                .iter()
                .map(|(key, value)| format!("{}: {}", key.to_string(), value.to_string()))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    pub fn get(&self, index: Value) -> Value {
        self.inner
            .get(&index)
            .map_or(Value::Null, |value| value.clone())
    }

    pub fn set(&mut self, index: Value, value: Value) {
        self.inner.insert(index.clone(), value.clone());
    }

    pub fn reserve(&mut self, capacity: usize) {
        self.inner.reserve(capacity);
    }

    pub fn concat(&self, other: SharedMutable<Self>) -> SharedMutable<Self> {
        let mut result = Self::new();
        result.reserve(self.count().max(other.borrow().count()));

        for (key, value) in &self.inner {
            result.set(key.clone(), value.clone());
        }
        for (key, value) in &other.borrow().inner {
            result.set(key.clone(), value.clone());
        }

        SharedMutable::new(result)
    }

    pub fn count(&self) -> usize {
        self.inner.len()
    }
}
