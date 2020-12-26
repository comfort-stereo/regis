use std::fmt::{Display, Formatter, Result};
use std::hash::{Hash, Hasher};

use crate::shared::{SharedImmutable, SharedMutable};

use super::dict::Dict;
use super::function::Function;
use super::list::List;

#[derive(Debug)]
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(SharedImmutable<String>),
    List(SharedMutable<List>),
    Dict(SharedMutable<Dict>),
    Function(SharedImmutable<Function>),
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Self::Null => Self::Null,
            Self::Boolean(value) => Self::Boolean(*value),
            Self::Integer(value) => Self::Integer(*value),
            Self::Float(value) => Self::Float(*value),
            Self::String(value) => Self::String(value.clone()),
            Self::List(value) => Self::List(value.clone()),
            Self::Dict(value) => Self::Dict(value.clone()),
            Self::Function(value) => Self::Function(value.clone()),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Boolean(left), Self::Boolean(right)) => left == right,
            (Self::Integer(left), Self::Integer(right)) => left == right,
            (Self::Float(left), Self::Float(right)) => left == right,
            (Self::Integer(left), Self::Float(right)) => (*left as f64) == *right,
            (Self::Float(left), Self::Integer(right)) => *left == (*right as f64),
            (Self::String(left), Self::String(right)) => *left == *right,
            (Self::List(left), Self::List(right)) => left == right,
            (Self::Dict(left), Self::Dict(right)) => left == right,
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
            Self::Integer(value) => value.hash(state),
            Self::Float(value) => (*value as i64).hash(state),
            Self::String(value) => value.hash(state),
            Self::List(value) => value.hash(state),
            Self::Dict(value) => value.hash(state),
            Self::Function(value) => value.hash(state),
        };
    }
}

impl Value {
    pub fn type_of(&self) -> ValueType {
        match self {
            Self::Null => ValueType::Null,
            Self::Boolean(..) => ValueType::Boolean,
            Self::Integer(..) => ValueType::Integer,
            Self::Float(..) => ValueType::Float,
            Self::String(..) => ValueType::String,
            Self::List(value) => value.borrow().type_of(),
            Self::Dict(value) => value.borrow().type_of(),
            Self::Function(value) => value.type_of(),
        }
    }

    pub fn to_boolean(&self) -> bool {
        match self {
            Self::Null => false,
            Self::Boolean(value) => *value,
            Self::Integer(value) => *value != 0,
            Self::Float(value) => *value != 0.0,
            Self::String(..) => true,
            Self::List(value) => value.borrow().to_boolean(),
            Self::Dict(value) => value.borrow().to_boolean(),
            Self::Function(value) => value.to_boolean(),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Null => "null".into(),
            Self::Boolean(value) => value.to_string(),
            Self::Integer(value) => value.to_string(),
            Self::Float(value) => value.to_string(),
            Self::String(value) => (**value).clone(),
            Self::List(value) => value.borrow().to_string(),
            Self::Dict(value) => value.borrow().to_string(),
            Self::Function(value) => value.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ValueType {
    Null,
    Boolean,
    Integer,
    Float,
    String,
    List,
    Dict,
    Function,
}

impl Display for ValueType {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        match self {
            Self::Null => write!(formatter, "null"),
            Self::Boolean => write!(formatter, "boolean"),
            Self::Integer => write!(formatter, "integer"),
            Self::Float => write!(formatter, "float"),
            Self::String => write!(formatter, "string"),
            Self::List => write!(formatter, "list"),
            Self::Dict => write!(formatter, "dict"),
            Self::Function => write!(formatter, "function"),
        }
    }
}
