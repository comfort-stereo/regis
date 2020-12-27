mod base;
mod environment;
mod expression;
mod marker;
mod operator;
mod statement;

use std::collections::{BTreeMap, HashSet};

use crate::shared::SharedMutable;

use super::instruction::Instruction;
use super::Bytecode;

use environment::Environment;
use marker::Marker;

#[derive(Debug)]
pub struct Builder {
    instructions: Vec<Instruction>,
    markers: BTreeMap<usize, HashSet<Marker>>,
    environment: SharedMutable<Environment>,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            markers: BTreeMap::new(),
            environment: Environment::new().into(),
        }
    }

    pub fn new_with_parent_environment(parent_environment: SharedMutable<Environment>) -> Self {
        Self {
            instructions: Vec::new(),
            markers: BTreeMap::new(),
            environment: Environment::new_with_parent(parent_environment).into(),
        }
    }

    pub fn last(&self) -> usize {
        self.instructions.len() - 1
    }

    pub fn end(&self) -> usize {
        self.instructions.len()
    }

    pub fn set(&mut self, line: usize, instruction: Instruction) {
        self.instructions[line] = instruction;
    }

    pub fn add(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    pub fn blank(&mut self) -> usize {
        self.add(Instruction::Blank);
        self.last()
    }

    fn environment(&self) -> &SharedMutable<Environment> {
        &self.environment
    }

    pub fn mark(&mut self, line: usize, marker: Marker) {
        self.markers.entry(line).or_insert_with(HashSet::new);
        self.markers
            .get_mut(&line)
            .map(|group| group.insert(marker));
    }

    pub fn has_marker(&self, line: usize, marker: Marker) -> bool {
        self.markers
            .get(&line)
            .map(|group| group.contains(&marker))
            .unwrap_or(false)
    }

    pub fn build(mut self) -> Bytecode {
        self.finalize();
        Bytecode::new(
            self.instructions,
            self.environment.borrow().parameters().clone(),
            self.environment.borrow().variables().clone(),
        )
    }

    fn finalize(&mut self) {
        for line in 0..=self.instructions.len() {
            if self.has_marker(line, Marker::Break) {
                self.finalize_break(line);
            }
            if self.has_marker(line, Marker::Continue) {
                self.finalize_continue(line);
            }
        }
    }

    fn finalize_break(&mut self, line: usize) {
        assert!(self.has_marker(line, Marker::Break));

        let mut depth = 0;
        for current in line..=self.instructions.len() {
            if self.has_marker(current, Marker::LoopStart) {
                depth += 1;
            } else if self.has_marker(current, Marker::LoopEnd) {
                if depth == 0 {
                    self.set(line, Instruction::Jump(current));
                    return;
                }

                depth -= 1;
            }
        }
    }

    fn finalize_continue(&mut self, line: usize) {
        assert!(self.has_marker(line, Marker::Continue));
        let mut depth = 0;
        for current in (0..=line).rev() {
            if self.has_marker(current, Marker::LoopEnd) {
                depth += 1;
            } else if self.has_marker(current, Marker::LoopStart) {
                if depth == 0 {
                    self.set(line, Instruction::Jump(current));
                    break;
                }

                depth -= 1
            }
        }
    }
}
