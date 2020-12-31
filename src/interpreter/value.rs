use std::fmt::{Display, Formatter, Result as FormatResult};
use std::hash::{Hash, Hasher};

use crate::shared::{SharedImmutable, SharedMutable};

use super::function::Function;
use super::list::List;
use super::object::Object;

#[derive(Debug)]
pub enum Value {
    Null,
    Boolean(bool),
    Int(i64),
    Float(f64),
    String(SharedImmutable<String>),
    List(SharedMutable<List>),
    Object(SharedMutable<Object>),
    Function(SharedImmutable<Function>),
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Self::Null => Self::Null,
            Self::Boolean(value) => Self::Boolean(*value),
            Self::Int(value) => Self::Int(*value),
            Self::Float(value) => Self::Float(*value),
            Self::String(value) => Self::String(value.clone()),
            Self::List(value) => Self::List(value.clone()),
            Self::Object(value) => Self::Object(value.clone()),
            Self::Function(value) => Self::Function(value.clone()),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Boolean(left), Self::Boolean(right)) => left == right,
            (Self::Int(left), Self::Int(right)) => left == right,
            (Self::Float(left), Self::Float(right)) => left == right,
            (Self::Int(left), Self::Float(right)) => (*left as f64) == *right,
            (Self::Float(left), Self::Int(right)) => *left == (*right as f64),
            (Self::String(left), Self::String(right)) => *left == *right,
            (Self::List(left), Self::List(right)) => left == right,
            (Self::Object(left), Self::Object(right)) => left == right,
            (Self::Function(left), Self::Function(right)) => left == right,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Null => 0.hash(state),
            Self::Boolean(value) => value.hash(state),
            Self::Int(value) => value.hash(state),
            Self::Float(value) => (*value as i64).hash(state),
            Self::String(value) => value.hash(state),
            Self::List(value) => value.hash(state),
            Self::Object(value) => value.hash(state),
            Self::Function(value) => value.hash(state),
        };
    }
}

impl Display for Value {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        write!(
            formatter,
            "{}",
            match self {
                Self::Null => "null".into(),
                Self::Boolean(value) => value.to_string(),
                Self::Int(value) => value.to_string(),
                Self::Float(value) => value.to_string(),
                Self::String(value) => (**value).clone(),
                Self::List(value) => value.borrow().to_string(),
                Self::Object(value) => value.borrow().to_string(),
                Self::Function(value) => value.to_string(),
            }
        )
    }
}

impl Value {
    pub fn type_of(&self) -> ValueType {
        match self {
            Self::Null => ValueType::Null,
            Self::Boolean(..) => ValueType::Boolean,
            Self::Int(..) => ValueType::Int,
            Self::Float(..) => ValueType::Float,
            Self::String(..) => ValueType::String,
            Self::List(value) => value.borrow().type_of(),
            Self::Object(value) => value.borrow().type_of(),
            Self::Function(value) => value.type_of(),
        }
    }

    pub fn to_boolean(&self) -> bool {
        match self {
            Self::Null => false,
            Self::Boolean(value) => *value,
            Self::Int(value) => *value != 0,
            Self::Float(value) => *value != 0.0,
            Self::String(..) => true,
            Self::List(value) => value.borrow().to_boolean(),
            Self::Object(value) => value.borrow().to_boolean(),
            Self::Function(value) => value.to_boolean(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ValueType {
    Null,
    Boolean,
    Int,
    Float,
    String,
    List,
    Object,
    Function,
}

impl Display for ValueType {
    fn fmt(&self, formatter: &mut Formatter) -> FormatResult {
        match self {
            Self::Null => write!(formatter, "null"),
            Self::Boolean => write!(formatter, "boolean"),
            Self::Int => write!(formatter, "int"),
            Self::Float => write!(formatter, "float"),
            Self::String => write!(formatter, "string"),
            Self::List => write!(formatter, "list"),
            Self::Object => write!(formatter, "object"),
            Self::Function => write!(formatter, "function"),
        }
    }
}
