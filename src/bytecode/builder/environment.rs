use std::collections::HashMap;

use crate::bytecode::variable::VariableVariant;
use crate::shared::{SharedImmutable, SharedMutable};

use super::super::variable::{Parameter, Variable};

type Scope = HashMap<SharedImmutable<String>, usize>;

#[derive(Debug)]
pub struct Environment {
    parent: Option<SharedMutable<Self>>,
    scopes: Vec<Scope>,
    parameters: Vec<Parameter>,
    variables: Vec<Variable>,
}

impl Default for Environment {
    fn default() -> Self {
        Environment::new()
    }
}

impl Environment {
    pub fn new() -> Self {
        Self {
            parent: None,
            scopes: vec![Scope::new()],
            parameters: Vec::new(),
            variables: Vec::new(),
        }
    }

    pub fn new_with_parent(parent: SharedMutable<Self>) -> Self {
        Self {
            parent: Some(parent),
            ..Self::new()
        }
    }

    pub fn parameters(&self) -> &Vec<Parameter> {
        &self.parameters
    }

    pub fn variables(&self) -> &Vec<Variable> {
        &self.variables
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
        let address = self.parameters.len();
        self.parameters.push(parameter);
        self.scopes
            .last_mut()
            .expect("There was no scope to add a parameter to.")
            .insert(name, address);

        address
    }

    pub fn add_variable(&mut self, variable: Variable) -> usize {
        let name = variable.name.clone();
        let address = self.parameters.len() + self.variables.len();
        self.variables.push(variable);
        self.scopes
            .last_mut()
            .expect("There was no scope to add a variable to.")
            .insert(name, address);

        address
    }

    pub fn get_or_capture_variable_address(&mut self, name: &SharedImmutable<String>) -> usize {
        self.try_get_or_capture_variable_address(name)
            .unwrap_or_else(|| panic!("Variable '{}' was not found in scope.", name))
    }

    fn try_get_or_capture_variable_address(
        &mut self,
        name: &SharedImmutable<String>,
    ) -> Option<usize> {
        for scope in self.scopes.iter().rev() {
            if let Some(address) = scope.get(name) {
                return Some(*address);
            }
        }

        if let Some(parent) = &self.parent {
            let offset = parent
                .borrow_mut()
                .try_get_capture_variable_address(name, 0);

            if let Some(offset) = offset {
                return Some(self.add_variable(Variable {
                    name: name.clone(),
                    variant: VariableVariant::Capture { offset },
                }));
            }
        }

        None
    }

    fn try_get_capture_variable_address(
        &mut self,
        name: &SharedImmutable<String>,
        offset: usize,
    ) -> Option<usize> {
        let end = offset + self.variables.len();
        for scope in self.scopes.iter().rev() {
            if let Some(address) = scope.get(name) {
                return Some(end - address);
            }
        }

        if let Some(parent) = &self.parent {
            return parent
                .borrow_mut()
                .try_get_capture_variable_address(name, end);
        }

        None
    }
}
