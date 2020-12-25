use std::fmt::{Debug, Formatter, Result as FormatResult};

use crate::function::Function;
use crate::shared::SharedImmutable;

#[derive(Debug, Clone)]
pub enum Instruction {
    Blank,
    Pop,
    Duplicate,
    DuplicateTop(usize),
    JumpIf(usize),
    JumpUnless(usize),
    Jump(usize),
    Return,
    IsNull,
    PushNull,
    PushBoolean(bool),
    PushNumber(f64),
    PushString(SharedImmutable<String>),
    PushVariable(usize),
    AssignVariable(usize),
    CreateList(usize),
    CreateDict(usize),
    CreateFunction(SharedImmutable<Function>),
    Call(usize),
    BinaryAdd,
    BinaryDiv,
    BinaryMul,
    BinarySub,
    BinaryGt,
    BinaryLt,
    BinaryGte,
    BinaryLte,
    BinaryEq,
    BinaryNeq,
    BinaryPush,
    GetIndex,
    SetIndex,
    Echo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Marker {
    LoopStart,
    LoopEnd,
    Break,
    Continue,
}

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
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            variable_count: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    pub fn get(&self, line: usize) -> &Instruction {
        &self.instructions[line]
    }

    pub fn set(&mut self, line: usize, instruction: Instruction) {
        self.instructions[line] = instruction;
    }

    pub fn add(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    pub fn add_variable(&mut self) -> usize {
        let count = self.variable_count;
        self.variable_count += 1;
        count
    }

    pub fn variable_count(&self) -> usize {
        self.variable_count
    }
}
