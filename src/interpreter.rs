mod error;

pub use error::InterpreterError;

use crate::ast::base::AstModule;
use crate::ast::Ast;
use crate::bytecode::Bytecode;
use crate::vm::Vm;

pub struct Interpreter {
    vm: Vm,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self { vm: Vm::new() }
    }

    pub fn run_module(&mut self, source: &str) -> Result<(), InterpreterError> {
        let ast = match Ast::<AstModule>::parse_module(&source) {
            Ok(ast) => ast,
            Err(error) => {
                return Err(InterpreterError::ParseError(error));
            }
        };

        let bytecode = Bytecode::compile_module(&ast);
        self.vm
            .run_module(&bytecode)
            .map_err(InterpreterError::VmError)
    }
}
