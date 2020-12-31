use std::collections::HashMap;

use indexmap::IndexSet;

use crate::path::CanonicalPath;
use crate::shared::SharedImmutable;

use super::variable::GlobalLocation;
use super::{
    ExportLocation, Parameter, StackLocation, Variable, VariableLocation, VariableVariant,
};

type Scope = HashMap<SharedImmutable<String>, usize>;

#[derive(Debug, Clone)]
pub struct Environment {
    path: CanonicalPath,
    parent: Option<Box<Self>>,
    globals: IndexSet<SharedImmutable<String>>,
    exports: IndexSet<SharedImmutable<String>>,
    scopes: Vec<Scope>,
    parameters: Vec<Parameter>,
    variables: Vec<Variable>,
}

impl Environment {
    pub fn new(path: CanonicalPath) -> Self {
        Self {
            path,
            parent: None,
            parameters: Vec::new(),
            variables: Vec::new(),
            globals: IndexSet::new(),
            exports: IndexSet::new(),
            scopes: vec![Scope::new()],
        }
    }

    pub fn path(&self) -> &CanonicalPath {
        &self.path
    }

    pub fn for_function(&self) -> Self {
        Self {
            parent: Some(self.clone().into()),
            globals: self.globals.clone(),
            ..Self::new(self.path.clone())
        }
    }

    pub fn for_module(&self, path: CanonicalPath) -> Self {
        Self {
            globals: self.globals.clone(),
            ..Self::new(path)
        }
    }

    pub fn parameters(&self) -> &Vec<Parameter> {
        &self.parameters
    }

    pub fn variables(&self) -> &Vec<Variable> {
        &self.variables
    }

    pub fn frame_size(&self) -> usize {
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
        let address = self.frame_size();
        self.parameters.push(parameter);
        self.scopes.first_mut().unwrap().insert(name, address);

        address
    }

    pub fn add_variable(&mut self, variable: Variable) -> usize {
        let name = variable.name.clone();
        let address = self.frame_size();
        self.variables.push(variable);
        self.scopes.last_mut().unwrap().insert(name, address);

        address
    }

    pub fn add_global(&mut self, name: SharedImmutable<String>) -> usize {
        self.globals.insert(name.clone());
        self.globals.get_index_of(&name).unwrap()
    }

    pub fn register_local_variable(&mut self, name: SharedImmutable<String>) -> usize {
        let scope = self.scopes.last_mut().unwrap();
        if let Some(address) = scope.get(&name) {
            *address
        } else {
            self.add_variable(Variable {
                name,
                variant: VariableVariant::Local,
            })
        }
    }

    pub fn register_export_variable(&mut self, name: SharedImmutable<String>) {
        self.exports.insert(name);
    }

    pub fn register_global_variable(&mut self, name: SharedImmutable<String>) {
        self.globals.insert(name);
    }

    pub fn get_variable_location(
        &self,
        name: &SharedImmutable<String>,
    ) -> Option<VariableLocation> {
        fn get_local_variable_address(
            environment: &Environment,
            name: &SharedImmutable<String>,
        ) -> Option<usize> {
            environment
                .scopes
                .iter()
                .rev()
                .filter_map(|scope| scope.get(name))
                .next()
                .cloned()
        }

        // Check to see if it's a local variable the current environment.
        if let Some(address) = get_local_variable_address(self, name) {
            return Some(VariableLocation::Stack(StackLocation {
                ascend: 0,
                address,
            }));
        }

        // Check to see if it's a local variable in a containing environment.
        {
            let mut ascend = 1;
            let mut current = self.parent.as_ref();
            while let Some(ancestor) = current {
                if let Some(address) = get_local_variable_address(&ancestor, name) {
                    return Some(VariableLocation::Stack(StackLocation { ascend, address }));
                }

                ascend += 1;
                current = ancestor.parent.as_ref();
            }
        }

        // Check to see if it's an exported variable from the current environment.
        if self.exports.contains(name) {
            return Some(VariableLocation::Export(ExportLocation {
                path: self.path.clone(),
                export: name.clone(),
            }));
        }

        // Check to see if it's an exported variable in a containing environment.
        {
            let mut current = self.parent.as_ref();

            while let Some(ancestor) = current {
                if ancestor.exports.contains(name) {
                    return Some(VariableLocation::Export(ExportLocation {
                        path: ancestor.path.clone(),
                        export: name.clone(),
                    }));
                }

                current = ancestor.parent.as_ref();
            }
        }

        // Check to see if the variable is global.
        if let Some(address) = self.globals.get_index_of(name) {
            return Some(VariableLocation::Global(GlobalLocation { address }));
        }

        None
    }
}
