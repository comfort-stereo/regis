mod builder;
mod environment;
mod instruction;
mod module;
mod procedure;
mod variable;

use std::fmt::{Debug, Formatter, Result as FormatResult};

pub use self::builder::Builder;
pub use self::environment::Environment;
pub use self::instruction::Instruction;
pub use self::module::Module;
pub use self::procedure::Procedure;
pub use self::variable::{
    ExportLocation, Parameter, StackLocation, Variable, VariableLocation, VariableVariant,
};

use crate::source::Span;

pub struct Bytecode {
    instructions: Vec<Instruction>,
    spans: Vec<Span>,
}

impl Debug for Bytecode {
    fn fmt(&self, formatter: &mut Formatter) -> FormatResult {
        formatter
            .debug_map()
            .entries(self.instructions.iter().enumerate())
            .finish()
    }
}

impl Bytecode {
    pub fn new(instructions: Vec<Instruction>, spans: Vec<Span>) -> Self {
        Self {
            instructions,
            spans,
        }
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    pub fn spans(&self) -> &[Span] {
        &self.spans
    }
}
