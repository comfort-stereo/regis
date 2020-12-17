use crate::list::List;
use crate::shared::Shared;
use crate::value_type::ValueType;

#[derive(Debug)]
pub enum Value {
    Null,
    Boolean(bool),
    Number(f64),
    String(Shared<String>),
    List(Shared<List>),
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::Null => Value::Null,
            Value::Boolean(value) => Value::Boolean(*value),
            Value::Number(value) => Value::Number(*value),
            Value::String(value) => Value::String(value.clone()),
            Value::List(list) => Value::List(list.clone()),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Boolean(left), Value::Boolean(right)) => left == right,
            (Value::Number(left), Value::Number(right)) => left == right,
            (Value::String(left), Value::String(right)) => *left.borrow() == *right.borrow(),
            (Value::List(left), Value::List(right)) => left == right,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl Value {
    pub fn type_of(&self) -> ValueType {
        match self {
            Value::Null => ValueType::Null,
            Value::Boolean(..) => ValueType::Boolean,
            Value::Number(..) => ValueType::Number,
            Value::String(..) => ValueType::String,
            Value::List(list) => list.borrow().type_of(),
        }
    }

    pub fn to_boolean(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Boolean(value) => *value,
            Value::Number(value) => *value != 0.0,
            Value::String(..) => true,
            Value::List(list) => list.borrow().to_boolean(),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Value::Null => "null".into(),
            Value::Boolean(value) => value.to_string(),
            Value::Number(value) => value.to_string(),
            Value::String(value) => value.borrow().clone(),
            Value::List(list) => list.borrow().to_string(),
        }
    }
}
