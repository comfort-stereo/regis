use std::hash::{Hash, Hasher};

use crate::dict::Dict;
use crate::function::Function;
use crate::list::List;
use crate::shared::{SharedImmutable, SharedMutable};
use crate::value_type::ValueType;

#[derive(Debug)]
pub enum Value {
    Null,
    Boolean(bool),
    Number(f64),
    String(SharedImmutable<String>),
    List(SharedMutable<List>),
    Dict(SharedMutable<Dict>),
    Function(SharedImmutable<Function>),
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::Null => Value::Null,
            Value::Boolean(value) => Value::Boolean(*value),
            Value::Number(value) => Value::Number(*value),
            Value::String(value) => Value::String(value.clone()),
            Value::List(value) => Value::List(value.clone()),
            Value::Dict(value) => Value::Dict(value.clone()),
            Value::Function(value) => Value::Function(value.clone()),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Boolean(left), Value::Boolean(right)) => left == right,
            (Value::Number(left), Value::Number(right)) => left == right,
            (Value::String(left), Value::String(right)) => *left == *right,
            (Value::List(left), Value::List(right)) => left == right,
            (Value::Dict(left), Value::Dict(right)) => left == right,
            (Value::Function(left), Value::Function(right)) => left == right,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Null => 0.hash(state),
            Value::Boolean(value) => value.hash(state),
            Value::Number(value) => (*value as i64).hash(state),
            Value::String(value) => value.hash(state),
            Value::List(value) => value.hash(state),
            Value::Dict(value) => value.hash(state),
            Value::Function(value) => value.hash(state),
        };
    }
}

impl Value {
    pub fn type_of(&self) -> ValueType {
        match self {
            Value::Null => ValueType::Null,
            Value::Boolean(..) => ValueType::Boolean,
            Value::Number(..) => ValueType::Number,
            Value::String(..) => ValueType::String,
            Value::List(value) => value.borrow().type_of(),
            Value::Dict(value) => value.borrow().type_of(),
            Value::Function(value) => value.type_of(),
        }
    }

    pub fn to_boolean(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Boolean(value) => *value,
            Value::Number(value) => *value != 0.0,
            Value::String(..) => true,
            Value::List(value) => value.borrow().to_boolean(),
            Value::Dict(value) => value.borrow().to_boolean(),
            Value::Function(value) => value.to_boolean(),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Value::Null => "null".into(),
            Value::Boolean(value) => value.to_string(),
            Value::Number(value) => value.to_string(),
            Value::String(value) => (**value).clone(),
            Value::List(value) => value.borrow().to_string(),
            Value::Dict(value) => value.borrow().to_string(),
            Value::Function(value) => value.to_string(),
        }
    }
}
