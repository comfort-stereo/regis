mod base;
mod expr;
mod marker;
mod operator;
mod stmt;

use std::collections::{BTreeMap, HashSet};

use crate::ast::NodeInfo;
use crate::shared::SharedImmutable;
use crate::source::Span;

use super::environment::Environment;
use super::instruction::Instruction;
use super::variable::GlobalLocation;
use super::{Bytecode, Variable, VariableLocation, VariableVariant};

use marker::Marker;

#[derive(Debug)]
pub struct Builder<'environment> {
    instructions: Vec<Instruction>,
    spans: Vec<Span>,
    markers: BTreeMap<usize, HashSet<Marker>>,
    environment: &'environment mut Environment,
}

impl<'environment> Builder<'environment> {
    pub fn new(environment: &'environment mut Environment) -> Self {
        Self {
            instructions: Vec::new(),
            spans: Vec::new(),
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

    pub fn set(&mut self, line: usize, instruction: Instruction, origin: &NodeInfo) {
        self.set_with_span(line, instruction, *origin.span());
    }

    pub fn add(&mut self, instruction: Instruction, origin: &NodeInfo) {
        self.add_with_span(instruction, *origin.span());
    }

    pub fn blank(&mut self, origin: &NodeInfo) -> usize {
        self.add(Instruction::Blank, origin);
        self.last()
    }

    pub fn set_with_span(&mut self, line: usize, instruction: Instruction, span: Span) {
        self.instructions[line] = instruction;
        self.spans[line] = span;
    }

    pub fn add_with_span(&mut self, instruction: Instruction, span: Span) {
        self.instructions.push(instruction);
        self.spans.push(span);
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

    pub fn emit_variable_assign_instruction(
        &mut self,
        name: &SharedImmutable<String>,
        origin: &NodeInfo,
    ) {
        self.emit_variable_instruction(name, true, origin);
    }

    pub fn emit_variable_push_instruction(
        &mut self,
        name: &SharedImmutable<String>,
        origin: &NodeInfo,
    ) {
        self.emit_variable_instruction(name, false, origin);
    }

    fn emit_variable_instruction(
        &mut self,
        name: &SharedImmutable<String>,
        assign: bool,
        origin: &NodeInfo,
    ) {
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

        self.add(instruction, &origin);
    }

    pub fn build(mut self) -> Bytecode {
        self.finalize();
        Bytecode::new(self.instructions, self.spans)
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
                    self.set_with_span(line, Instruction::Jump(current), self.spans[line]);
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
                    self.set_with_span(line, Instruction::Jump(current), self.spans[line]);
                    break;
                }

                depth -= 1
            }
        }
    }
}
