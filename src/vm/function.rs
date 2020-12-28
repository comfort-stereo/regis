use std::fmt::{Debug, Display, Formatter, Result as FormatResult};
use std::hash::{Hash, Hasher};

use crate::bytecode::{Bytecode, Procedure};
use crate::shared::{SharedImmutable, SharedMutable};

use super::closure::Capture;
use super::rid::rid;
use super::value::ValueType;

pub struct Function {
    id: usize,
    procedure: SharedImmutable<Procedure>,
    captures: Vec<SharedMutable<Capture>>,
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Function {}

impl Hash for Function {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl Display for Function {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        match self.name() {
            Some(name) => write!(formatter, "<function:{}>", *name),
            None => write!(formatter, "<function>"),
        }
    }
}

impl Debug for Function {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        write!(
            formatter,
            "fn {}() [{:?}]",
            self.name().unwrap_or_else(|| String::from("").into()),
            self.captures
                .iter()
                .map(|capture| capture.borrow().position)
                .collect::<Vec<_>>(),
        )
    }
}

impl Function {
    pub fn new(
        procedure: SharedImmutable<Procedure>,
        captures: Vec<SharedMutable<Capture>>,
    ) -> Self {
        Self {
            id: rid(),
            procedure,
            captures,
        }
    }

    pub fn type_of(&self) -> ValueType {
        ValueType::Function
    }

    pub fn to_boolean(&self) -> bool {
        true
    }

    pub fn name(&self) -> Option<SharedImmutable<String>> {
        self.procedure.name().clone()
    }

    pub fn bytecode(&self) -> &Bytecode {
        &self.procedure.bytecode()
    }

    pub fn captures(&self) -> &Vec<SharedMutable<Capture>> {
        &self.captures
    }
}
