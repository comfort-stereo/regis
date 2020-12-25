use std::collections::{BTreeMap, HashMap, HashSet};

use crate::shared::SharedImmutable;

use super::super::bytecode::Bytecode;
use super::super::instruction::Instruction;
use super::marker::Marker;

type Scope = HashMap<SharedImmutable<String>, usize>;

#[derive(Debug)]
pub struct Builder {
    bytecode: Bytecode,
    markers: BTreeMap<usize, HashSet<Marker>>,
    scopes: Vec<Scope>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            bytecode: Bytecode::new(),
            markers: BTreeMap::new(),
            scopes: vec![HashMap::new()],
        }
    }

    pub fn last(&self) -> usize {
        self.bytecode.len() - 1
    }

    pub fn end(&self) -> usize {
        self.bytecode.len()
    }

    pub fn set(&mut self, line: usize, instruction: Instruction) {
        self.bytecode.set(line, instruction);
    }

    pub fn add(&mut self, instruction: Instruction) {
        self.bytecode.add(instruction);
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
        let address = self.bytecode.add_variable();
        self.scopes
            .last_mut()
            .expect("There was no scope to add a variable to.")
            .insert(name.clone(), address);
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

    pub fn build(mut self) -> Bytecode {
        self.finalize();
        self.bytecode
    }

    pub fn mark(&mut self, line: usize, marker: Marker) {
        if !self.markers.contains_key(&line) {
            self.markers.insert(line, HashSet::new());
        }

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

    fn finalize(&mut self) {
        for line in 0..=self.bytecode.len() {
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
        for current in line..=self.bytecode.len() {
            if self.has_marker(current, Marker::LoopStart) {
                depth += 1;
            } else if self.has_marker(current, Marker::LoopEnd) {
                if depth == 0 {
                    self.bytecode.set(line, Instruction::Jump(current));
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
