use std::fmt::{Display, Formatter, Result};

#[derive(Debug, PartialEq, Eq)]
pub enum ValueType {
    Null,
    Boolean,
    Number,
    String,
    List,
    Dict,
    Function,
}

impl Display for ValueType {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        match self {
            ValueType::Null => write!(formatter, "null"),
            ValueType::Boolean => write!(formatter, "boolean"),
            ValueType::Number => write!(formatter, "number"),
            ValueType::String => write!(formatter, "string"),
            ValueType::List => write!(formatter, "list"),
            ValueType::Dict => write!(formatter, "dict"),
            ValueType::Function => write!(formatter, "function"),
        }
    }
}
