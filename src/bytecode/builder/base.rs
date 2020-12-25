use crate::ast::base::{AstBlock, AstModule};

use super::builder::Builder;

impl Builder {
    pub fn emit_module(&mut self, AstModule { statements, .. }: &AstModule) {
        self.push_scope();
        for statement in statements {
            self.emit_statement(statement);
        }
        self.pop_scope();
    }

    pub fn emit_block(&mut self, AstBlock { statements, .. }: &AstBlock) {
        self.push_scope();
        for statement in statements {
            self.emit_statement(&statement);
        }
        self.pop_scope();
    }
}
