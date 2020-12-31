mod base;
mod expression;
mod marker;
mod operator;
mod statement;

use std::collections::{BTreeMap, HashSet};

use crate::shared::SharedImmutable;

use super::environment::Environment;
use super::instruction::Instruction;
use super::variable::GlobalLocation;
use super::{Bytecode, Variable, VariableLocation, VariableVariant};

use marker::Marker;

#[derive(Debug)]
pub struct Builder<'environment> {
    instructions: Vec<Instruction>,
    markers: BTreeMap<usize, HashSet<Marker>>,
    environment: &'environment mut Environment,
}

impl<'environment> Builder<'environment> {
    pub fn new(environment: &'environment mut Environment) -> Self {
        Self {
            instructions: Vec::new(),
            markers: BTreeMap::new(),
            environment,
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

    pub fn emit_variable_assign_instruction(&mut self, name: &SharedImmutable<String>) {
        self.emit_variable_instruction(name, true);
    }

    pub fn emit_variable_push_instruction(&mut self, name: &SharedImmutable<String>) {
        self.emit_variable_instruction(name, false);
    }

    fn emit_variable_instruction(&mut self, name: &SharedImmutable<String>, assign: bool) {
        let location = self
            .environment
            .get_variable_location(name)
            .unwrap_or_else(|| panic!("No variable '{}' found.", name));

        let instruction = match location {
            VariableLocation::Stack(location) => {
                let address = if location.ascend == 0 {
                    // If the variable is in the current stack frame, use the local address.
                    location.address
                } else {
                    // If the variable is in a containing environment, add a capture variable
                    // pointing to its location and use the capture variable's local address.
                    self.environment.add_variable(Variable {
                        name: name.clone(),
                        variant: VariableVariant::Capture { location },
                    })
                };

                if assign {
                    Instruction::AssignVariable(address)
                } else {
                    Instruction::PushVariable(address)
                }
            }
            VariableLocation::Export(location) => {
                if assign {
                    Instruction::AssignExport(location.into())
                } else {
                    Instruction::PushExport(location.into())
                }
            }
            VariableLocation::Global(GlobalLocation { address }) => {
                if assign {
                    panic!("Global variables cannot be reassigned.");
                } else {
                    Instruction::PushGlobal(address)
                }
            }
        };

        self.add(instruction);
    }

    pub fn build(mut self) -> Bytecode {
        self.finalize();
        Bytecode::new(self.instructions)
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
