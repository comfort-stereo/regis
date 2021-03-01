mod builder;
mod environment;
mod instruction;
mod module;
mod procedure;
mod variable;

use std::fmt::{Debug, Formatter, Result as FormatResult};

pub use builder::Builder;
pub use environment::Environment;
pub use instruction::Instruction;
pub use module::Module;
pub use procedure::Procedure;
pub use variable::{
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
