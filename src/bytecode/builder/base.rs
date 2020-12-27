use crate::ast::base::{AstBlock, AstModule};

use super::Builder;

impl Builder {
    pub fn emit_module(&mut self, AstModule { statements, .. }: &AstModule) {
        self.environment().borrow_mut().push_scope();
        for statement in statements {
            self.emit_statement(statement);
        }
        self.environment().borrow_mut().pop_scope();
    }

    pub fn emit_block(&mut self, AstBlock { statements, .. }: &AstBlock) {
        self.environment().borrow_mut().push_scope();
        for statement in statements {
            self.emit_statement(&statement);
        }
        self.environment().borrow_mut().pop_scope();
    }
}
