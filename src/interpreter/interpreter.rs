use crate::ast::base::AstModule;
use crate::ast::Ast;
use crate::bytecode::compile_module;
use crate::vm::{Vm, VmError};

pub struct Interpreter {
    vm: Vm,
}

impl Interpreter {
    pub fn new() -> Self {
        Self { vm: Vm::new() }
    }

    pub fn run_module(&mut self, code: &str) -> Result<(), VmError> {
        let ast = match Ast::<AstModule>::parse_module(&code) {
            Ok(ast) => ast,
            Err(error) => {
                return Err(VmError::ParseError { error });
            }
        };

        let bytecode = compile_module(&ast);
        self.vm.run(&bytecode)
    }
}
