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
            Self::Null => write!(formatter, "null"),
            Self::Boolean => write!(formatter, "boolean"),
            Self::Number => write!(formatter, "number"),
            Self::String => write!(formatter, "string"),
            Self::List => write!(formatter, "list"),
            Self::Dict => write!(formatter, "dict"),
            Self::Function => write!(formatter, "function"),
        }
    }
}
