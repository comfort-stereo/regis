mod builder;
mod instruction;
mod procedure;
mod variable;

use std::fmt::{Debug, Formatter, Result as FormatResult};

use crate::ast::base::AstModule;
use crate::ast::Ast;

pub use builder::Builder;
pub use instruction::Instruction;
pub use procedure::Procedure;
pub use variable::{Parameter, Variable, VariableVariant};

pub struct Bytecode {
    instructions: Vec<Instruction>,
    parameters: Vec<Parameter>,
    variables: Vec<Variable>,
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
    pub fn new(
        instructions: Vec<Instruction>,
        parameters: Vec<Parameter>,
        variables: Vec<Variable>,
    ) -> Self {
        Self {
            instructions,
            parameters,
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

    pub fn parameters(&self) -> &Vec<Parameter> {
        &self.parameters
    }

    pub fn variables(&self) -> &Vec<Variable> {
        &self.variables
    }
}
