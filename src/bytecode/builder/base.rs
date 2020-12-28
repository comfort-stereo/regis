use crate::ast::base::{AstBlock, AstModule};
use crate::ast::statement::AstStatementVariant;

use super::super::instruction::Instruction;
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

    pub fn emit_function_block(&mut self, AstBlock { statements, .. }: &AstBlock) {
        for statement in statements {
            self.emit_statement(&statement);
        }

        if !statements
            .iter()
            .any(|statement| matches!(statement, AstStatementVariant::ReturnStatement(..)))
        {
            self.add(Instruction::PushNull);
        }
    }
}
