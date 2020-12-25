use std::hash::{Hash, Hasher};

use crate::compiler::bytecode::Bytecode;
use crate::oid::oid;
use crate::shared::SharedImmutable;
use crate::value_type::ValueType;

#[derive(Debug)]
pub struct Function {
    id: usize,
    name: Option<SharedImmutable<String>>,
    parameters: Vec<SharedImmutable<String>>,
    bytecode: SharedImmutable<Bytecode>,
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
    pub fn new(
        name: Option<SharedImmutable<String>>,
        parameters: Vec<SharedImmutable<String>>,
        bytecode: SharedImmutable<Bytecode>,
    ) -> Self {
        Self {
            id: oid(),
            name,
            parameters,
            bytecode,
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
        self.name.clone()
    }

    // pub fn parameters(&self) -> &Vec<SharedImmutable<String>> {
    //     &self.parameters
    // }

    pub fn bytecode(&self) -> &SharedImmutable<Bytecode> {
        &self.bytecode
    }
}
