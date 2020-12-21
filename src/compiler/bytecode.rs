use std::fmt::{Debug, Formatter, Result as FormatResult};

use crate::function::Function;
use crate::shared::SharedImmutable;

#[derive(Debug, Clone)]
pub enum Instruction {
    Blank,
    Pop,
    Duplicate,
    DuplicateTop(usize),
    PushScope,
    PopScope,
    IsNull,
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
    PushNull,
    PushBoolean(bool),
    PushNumber(f64),
    PushString(SharedImmutable<String>),
    CreateList(usize),
    CreateDict(usize),
    CreateFunction(SharedImmutable<Function>),
    Call(usize),
    PushVariable(SharedImmutable<String>),
    DeclareVariable(SharedImmutable<String>),
    AssignVariable(SharedImmutable<String>),
    JumpIf(usize),
    JumpUnless(usize),
    Jump(usize),
    Return,
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
}
