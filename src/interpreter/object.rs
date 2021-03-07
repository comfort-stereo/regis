use std::fmt::{Display, Formatter, Result as FormatResult};
use std::hash::{Hash, Hasher};

use indexmap::IndexMap;

use crate::shared::SharedMutable;

use super::rid::Rid;
use super::value::Value;
use super::ValueType;

#[derive(Debug)]
pub struct Object {
    id: Rid,
    inner: IndexMap<Value, Value>,
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Object {}

impl Hash for Object {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl Display for Object {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        write!(
            formatter,
            "{{ {} }}",
            self.inner
                .iter()
                .map(|(key, value)| format!("{}: {}", key.to_string(), value.to_string()))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl Object {
    pub fn new(id: Rid) -> Self {
        Self {
            id,
            inner: IndexMap::new(),
        }
    }

    pub fn type_of(&self) -> ValueType {
        ValueType::Object
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

    pub fn get(&self, index: &Value) -> Value {
        self.inner
            .get(index)
            .map_or(Value::Null, |value| value.clone())
    }

    pub fn set(&mut self, index: Value, value: Value) {
        self.inner.insert(index, value);
    }

    pub fn reserve(&mut self, capacity: usize) {
        self.inner.reserve(capacity);
    }

    pub fn concat(&self, other: &Self, id: Rid) -> SharedMutable<Self> {
        let mut result = Self::new(id);
        result.reserve(self.len().max(other.len()));

        for (key, value) in &self.inner {
            result.set(key.clone(), value.clone());
        }
        for (key, value) in &other.inner {
            result.set(key.clone(), value.clone());
        }

        result.into()
    }
}
