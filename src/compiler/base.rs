use crate::ast::base::{AstBlock, AstModule};

use super::builder::Builder;
use super::bytecode::Instruction;

impl Builder {
    pub fn emit_module(&mut self, module: &AstModule) {
        for statement in &module.statements {
            self.emit_statement(statement);
        }
    }

    pub fn emit_block(&mut self, block: &AstBlock) {
        self.add(Instruction::PushScope);
        for statement in &block.statements {
            self.emit_statement(&statement);
        }
        self.add(Instruction::PopScope);
    }

    pub fn emit_unscoped_block(&mut self, block: &AstBlock) {
        for statement in &block.statements {
            self.emit_statement(&statement);
        }
    }
}
