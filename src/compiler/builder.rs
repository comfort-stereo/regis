use std::collections::{BTreeMap, HashSet};

use super::bytecode::{Bytecode, Instruction, Marker};

#[derive(Debug)]
pub struct Builder {
    bytecode: Bytecode,
    markers: BTreeMap<usize, HashSet<Marker>>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            bytecode: Bytecode::new(),
            markers: BTreeMap::new(),
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
