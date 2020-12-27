mod builder;
mod instruction;
mod procedure;

use std::fmt::{Debug, Formatter, Result as FormatResult};

use crate::ast::base::AstModule;
use crate::ast::Ast;
use crate::shared::SharedImmutable;

pub use builder::Builder;
pub use instruction::Instruction;
pub use procedure::Procedure;

pub struct Bytecode {
    instructions: Vec<Instruction>,
    variables: Vec<SharedImmutable<String>>,
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
    pub fn new(instructions: Vec<Instruction>, variables: Vec<SharedImmutable<String>>) -> Self {
        Self {
            instructions,
            variables,
        }
    }

    pub fn compile_module(module: &Ast<AstModule>) -> Bytecode {
        let mut builder = Builder::new();
        builder.emit_module(module.root());
        builder.build()
    }

    pub fn instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }

    pub fn variables(&self) -> &Vec<SharedImmutable<String>> {
        &self.variables
    }
}
