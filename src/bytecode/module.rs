use crate::ast::Chunk;
use crate::source::CanonicalPath;

use super::environment::Environment;
use super::{Builder, Bytecode};

#[derive(Debug)]
pub struct Module {
    path: CanonicalPath,
    bytecode: Bytecode,
    environment: Environment,
}

impl Module {
    pub fn new(path: CanonicalPath, bytecode: Bytecode, environment: Environment) -> Self {
        Self {
            path,
            bytecode,
            environment,
        }
    }

    pub fn build(path: CanonicalPath, chunk: &Chunk, mut environment: Environment) -> Self {
        let mut builder = Builder::new(&mut environment);
        builder.emit_chunk(chunk);
        let bytecode = builder.build();

        Self::new(path, bytecode, environment)
    }

    pub fn path(&self) -> &CanonicalPath {
        &self.path
    }

    pub fn bytecode(&self) -> &Bytecode {
        &self.bytecode
    }

    pub fn environment(&self) -> &Environment {
        &self.environment
    }
}
