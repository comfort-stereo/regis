use std::fmt::{Debug, Formatter, Result as FormatResult};

use super::instruction::Instruction;

pub struct Bytecode {
    instructions: Vec<Instruction>,
    variable_count: usize,
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
    pub fn new(instructions: Vec<Instruction>, variable_count: usize) -> Self {
        Self {
            instructions,
            variable_count,
        }
    }

    pub fn size(&self) -> usize {
        self.instructions.len()
    }

    pub fn get(&self, line: usize) -> Option<&Instruction> {
        self.instructions.get(line)
    }

    pub fn variable_count(&self) -> usize {
        self.variable_count
    }
}
