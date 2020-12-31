use crate::shared::SharedImmutable;

use super::environment::Environment;
use super::Bytecode;

#[derive(Debug)]
pub struct Procedure {
    name: Option<SharedImmutable<String>>,
    bytecode: Bytecode,
    environment: Environment,
}

impl Procedure {
    pub fn new(
        name: Option<SharedImmutable<String>>,
        bytecode: Bytecode,
        environment: Environment,
    ) -> Self {
        Self {
            name,
            bytecode,
            environment,
        }
    }

    pub fn name(&self) -> Option<&SharedImmutable<String>> {
        self.name.as_ref()
    }

    pub fn bytecode(&self) -> &Bytecode {
        &self.bytecode
    }

    pub fn environment(&self) -> &Environment {
        &self.environment
    }
}
