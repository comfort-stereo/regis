use crate::ast::base::AstModule;
use crate::ast::Ast;
use crate::path::CanonicalPath;

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

    pub fn build(path: CanonicalPath, ast: &Ast<AstModule>, environment: Environment) -> Self {
        let mut environment_mut = environment;
        let mut builder = Builder::new(&mut environment_mut);
        builder.emit_module(ast.root());
        let bytecode = builder.build();

        Self::new(path, bytecode, environment_mut)
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
