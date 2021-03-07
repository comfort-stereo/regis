use std::fmt::{Debug, Display, Formatter, Result as FormatResult};
use std::hash::{Hash, Hasher};

use crate::bytecode::Procedure;
use crate::shared::SharedImmutable;

use super::{rid::Rid, value::ValueType};
use super::{ExternalProcedure, StackValue};

pub struct Function {
    id: Rid,
    procedure: ProcedureVariant,
    init: Box<[StackValue]>,
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
    pub fn new(id: Rid, procedure: ProcedureVariant) -> Self {
        Self {
            id,
            procedure,
            init: Box::new([]),
        }
    }

    pub fn with_init(id: Rid, procedure: ProcedureVariant, init: Box<[StackValue]>) -> Self {
        Self {
            init,
            ..Self::new(id, procedure)
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

    pub fn init(&self) -> &[StackValue] {
        &self.init
    }
}

pub enum ProcedureVariant {
    Internal(SharedImmutable<Procedure>),
    External(Box<ExternalProcedure>),
}
