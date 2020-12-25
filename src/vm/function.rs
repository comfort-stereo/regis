use std::hash::{Hash, Hasher};

use crate::bytecode::{Bytecode, Procedure};
use crate::shared::SharedImmutable;

use super::rid::rid;
use super::value::ValueType;

#[derive(Debug)]
pub struct Function {
    id: usize,
    procedure: SharedImmutable<Procedure>,
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

impl Function {
    pub fn new(procedure: SharedImmutable<Procedure>) -> Self {
        Self {
            id: rid(),
            procedure,
        }
    }

    pub fn type_of(&self) -> ValueType {
        ValueType::Function
    }

    pub fn to_boolean(&self) -> bool {
        true
    }

    pub fn to_string(&self) -> String {
        match self.name() {
            Some(name) => format!("<function:{}>", *name),
            None => "<function>".into(),
        }
    }

    pub fn name(&self) -> Option<SharedImmutable<String>> {
        self.procedure.name().clone()
    }

    pub fn bytecode(&self) -> &Bytecode {
        &self.procedure.bytecode()
    }
}
