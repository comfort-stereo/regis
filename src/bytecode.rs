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

pub struct Bytecode {
    instructions: Vec<Instruction>,
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
    pub fn new(instructions: Vec<Instruction>) -> Self {
        Self { instructions }
    }

    pub fn instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }
}
