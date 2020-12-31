use std::fmt::{Debug, Display, Formatter, Result as FormatResult};
use std::hash::{Hash, Hasher};

use crate::bytecode::Procedure;
use crate::shared::{SharedImmutable, SharedMutable};

use super::capture::Capture;
use super::rid::rid;
use super::value::ValueType;
use super::ExternalProcedure;

pub struct Function {
    id: usize,
    procedure: ProcedureVariant,
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
        let default = SharedImmutable::new(String::from(""));
        write!(
            formatter,
            "fn {} () {{ ... }}",
            self.name().unwrap_or(&default),
        )
    }
}

impl Function {
    pub fn new(procedure: ProcedureVariant) -> Self {
        Self {
            id: rid(),
            procedure,
            captures: Vec::new(),
        }
    }

    pub fn with_captures(
        procedure: ProcedureVariant,
        captures: Vec<SharedMutable<Capture>>,
    ) -> Self {
        Self {
            captures,
            ..Self::new(procedure)
        }
    }

    pub fn type_of(&self) -> ValueType {
        ValueType::Function
    }

    pub fn to_boolean(&self) -> bool {
        true
    }

    pub fn name(&self) -> Option<&SharedImmutable<String>> {
        match &self.procedure {
            ProcedureVariant::Internal(internal) => internal.name(),
            ProcedureVariant::External(external) => Some(external.name()),
        }
    }

    pub fn procedure(&self) -> &ProcedureVariant {
        &self.procedure
    }

    pub fn captures(&self) -> &Vec<SharedMutable<Capture>> {
        &self.captures
    }
}

pub enum ProcedureVariant {
    Internal(SharedImmutable<Procedure>),
    External(Box<ExternalProcedure>),
}
