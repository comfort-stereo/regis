use crate::ast::base::{AstBlock, AstModule};

use super::builder::Builder;
use super::bytecode::Instruction;

impl Builder {
    pub fn emit_module(&mut self, AstModule { statements, .. }: &AstModule) {
        for statement in statements {
            self.emit_statement(statement);
        }
    }

    pub fn emit_block(&mut self, AstBlock { statements, .. }: &AstBlock) {
        self.add(Instruction::PushScope);
        for statement in statements {
            self.emit_statement(&statement);
        }
        self.add(Instruction::PopScope);
    }

    pub fn emit_unscoped_block(&mut self, AstBlock { statements, .. }: &AstBlock) {
        for statement in statements {
            self.emit_statement(&statement);
        }
    }
}
