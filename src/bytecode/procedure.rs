use crate::shared::SharedImmutable;

use super::bytecode::Bytecode;

#[derive(Debug)]
pub struct Procedure {
    name: Option<SharedImmutable<String>>,
    parameters: Vec<SharedImmutable<String>>,
    bytecode: Bytecode,
}

impl Procedure {
    pub fn new(
        name: Option<SharedImmutable<String>>,
        parameters: Vec<SharedImmutable<String>>,
        bytecode: Bytecode,
    ) -> Self {
        Self {
            name,
            parameters,
            bytecode,
        }
    }

    pub fn name(&self) -> &Option<SharedImmutable<String>> {
        &self.name
    }

    // pub fn parameters(&self) -> &Vec<SharedImmutable<String>> {
    //     &self.parameters
    // }

    pub fn bytecode(&self) -> &Bytecode {
        &self.bytecode
    }
}
