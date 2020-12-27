mod base;
mod expression;
mod marker;
mod operator;
mod statement;

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::shared::SharedImmutable;

use super::instruction::Instruction;
use super::Bytecode;

use marker::Marker;

type Scope = HashMap<SharedImmutable<String>, usize>;

#[derive(Debug)]
pub struct Builder {
    instructions: Vec<Instruction>,
    variables: Vec<SharedImmutable<String>>,
    markers: BTreeMap<usize, HashSet<Marker>>,
    scopes: Vec<Scope>,
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
            variables: Vec::new(),
            markers: BTreeMap::new(),
            scopes: vec![HashMap::new()],
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

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub fn pop_scope(&mut self) -> Scope {
        self.scopes.pop().expect("There was no scope to pop.")
    }

    pub fn add_variable(&mut self, name: SharedImmutable<String>) -> usize {
        let address = self.variables.len();
        self.variables.push(name.clone());
        self.scopes
            .last_mut()
            .expect("There was no scope to add a variable to.")
            .insert(name, address);

        address
    }

    pub fn get_variable_address(&self, name: &SharedImmutable<String>) -> usize {
        for scope in self.scopes.iter().rev() {
            if let Some(address) = scope.get(name) {
                return *address;
            }
        }

        panic!("No variable '{}' was found in scope.", name);
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
        Bytecode::new(self.instructions, self.variables)
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
