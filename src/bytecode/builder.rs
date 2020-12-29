mod base;
mod expression;
mod marker;
mod operator;
mod statement;

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::shared::SharedImmutable;

use super::instruction::Instruction;
use super::{Bytecode, Parameter, Variable, VariableLocation, VariableVariant};

use marker::Marker;

type Scope = HashMap<SharedImmutable<String>, usize>;

#[derive(Debug)]
pub struct Builder<'parent> {
    parent: Option<&'parent Self>,
    instructions: Vec<Instruction>,
    markers: BTreeMap<usize, HashSet<Marker>>,
    scopes: Vec<Scope>,
    parameters: Vec<Parameter>,
    variables: Vec<Variable>,
}

impl<'parent> Default for Builder<'parent> {
    fn default() -> Builder<'parent> {
        Self::new()
    }
}

impl<'parent> Builder<'parent> {
    pub fn new() -> Self {
        Self {
            parent: None,
            parameters: Vec::new(),
            variables: Vec::new(),
            scopes: vec![Scope::new()],
            instructions: Vec::new(),
            markers: BTreeMap::new(),
        }
    }

    pub fn new_child(parent: &'parent Self) -> Self {
        Self {
            parent: Some(parent),
            instructions: Vec::new(),
            markers: BTreeMap::new(),
            ..Self::new()
        }
    }

    pub fn parameters(&self) -> &Vec<Parameter> {
        &self.parameters
    }

    pub fn variables(&self) -> &Vec<Variable> {
        &self.variables
    }

    pub fn call_size(&self) -> usize {
        self.parameters.len() + self.variables.len()
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

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() == 1 {
            panic!("Cannot pop last scope from environment.")
        }

        self.scopes.pop().unwrap();
    }

    pub fn add_parameter(&mut self, parameter: Parameter) -> usize {
        assert!(self.variables.is_empty());
        let name = parameter.name.clone();
        let address = self.call_size();
        self.parameters.push(parameter);
        self.scopes.last_mut().unwrap().insert(name, address);

        address
    }

    pub fn add_variable(&mut self, variable: Variable) -> usize {
        let name = variable.name.clone();
        let address = self.call_size();
        self.variables.push(variable);
        self.scopes.last_mut().unwrap().insert(name, address);

        address
    }

    pub fn get_or_add_scope_variable(&mut self, name: &SharedImmutable<String>) -> usize {
        let scope = self.scopes.last_mut().unwrap();
        if let Some(address) = scope.get(name) {
            *address
        } else {
            self.add_variable(Variable {
                name: name.clone(),
                variant: VariableVariant::Local,
            })
        }
    }

    pub fn get_or_capture_variable(&mut self, name: &SharedImmutable<String>) -> usize {
        let location = self
            .get_variable_location(name)
            .unwrap_or_else(|| panic!("No variable '{}' found in scope.", name));

        // If the variable is in the current call, return the local address.
        if location.ascend == 0 {
            return location.address;
        }

        // If the variable is in a containing environment, add a capture variable pointing to its
        // location and return the capture variable's address.
        self.add_variable(Variable {
            name: name.clone(),
            variant: VariableVariant::Capture { location },
        })
    }

    fn get_variable_location(&self, name: &SharedImmutable<String>) -> Option<VariableLocation> {
        let mut ascend = 0;
        let mut current = Some(self);

        while let Some(builder) = current {
            if let Some(address) = builder.get_local_variable_address(name) {
                return Some(VariableLocation { ascend, address });
            }

            ascend += 1;
            current = builder.parent;
        }

        None
    }

    fn get_local_variable_address(&self, name: &SharedImmutable<String>) -> Option<usize> {
        self.scopes
            .iter()
            .rev()
            .filter_map(|scope| scope.get(name))
            .next()
            .cloned()
    }

    pub fn build(mut self) -> Bytecode {
        self.finalize();
        Bytecode::new(self.instructions, self.parameters, self.variables)
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
