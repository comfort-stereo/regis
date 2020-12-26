use crate::ast::base::AstModule;
use crate::ast::Ast;
use crate::bytecode::compile_module;
use crate::vm::Vm;

use super::InterpreterError;

pub struct Interpreter {
    vm: Vm,
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

        let bytecode = compile_module(&ast);

        self.vm
            .run(&bytecode)
            .map_err(|error| InterpreterError::VmError(error))
    }
}
