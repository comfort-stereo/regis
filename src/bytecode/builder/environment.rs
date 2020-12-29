use std::collections::HashMap;

use crate::shared::{SharedImmutable, SharedMutable};

use super::super::variable::{Parameter, Variable, VariableLocation, VariableVariant};

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

    pub fn size(&self) -> usize {
        self.parameters.len() + self.variables.len()
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
        let address = self.size();
        self.parameters.push(parameter);
        self.scopes
            .last_mut()
            .expect("There was no scope to add a parameter to.")
            .insert(name, address);

        address
    }

    pub fn add_variable(&mut self, variable: Variable) -> usize {
        let name = variable.name.clone();
        let address = self.size();
        self.variables.push(variable);
        self.scopes
            .last_mut()
            .expect("There was no scope to add a variable to.")
            .insert(name, address);

        address
    }

    pub fn get_or_add_local_variable_address(&mut self, name: &SharedImmutable<String>) -> usize {
        let scope = self
            .scopes
            .last_mut()
            .expect("There was no scope to add a variable to.");

        if let Some(address) = scope.get(name) {
            *address
        } else {
            self.add_variable(Variable {
                name: name.clone(),
                variant: VariableVariant::Local,
            })
        }
    }

    pub fn get_scope_variable_address(&self, name: &SharedImmutable<String>) -> Option<usize> {
        let location = self.get_variable_location(name);
        if let Some(VariableLocation { ascend, address }) = location {
            if ascend == 0 {
                return Some(address);
            }
        }

        None
    }

    pub fn get_or_capture_variable_address(&mut self, name: &SharedImmutable<String>) -> usize {
        let location = self
            .get_variable_location(name)
            .unwrap_or_else(|| panic!("No local '{}' found in scope.", name));

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

    pub fn get_variable_location(
        &self,
        name: &SharedImmutable<String>,
    ) -> Option<VariableLocation> {
        self.get_variable_location_aux(name, 0)
    }

    fn get_variable_location_aux(
        &self,
        name: &SharedImmutable<String>,
        ascend: usize,
    ) -> Option<VariableLocation> {
        for scope in self.scopes.iter().rev() {
            if let Some(address) = scope.get(name) {
                return Some(VariableLocation {
                    ascend,
                    address: *address,
                });
            }
        }

        if let Some(parent) = &self.parent {
            parent.borrow().get_variable_location_aux(name, ascend + 1)
        } else {
            None
        }
    }
}
