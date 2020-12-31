use crate::ast::base::{AstBlock, AstModule};
use crate::ast::statement::{
    AstFunctionDeclarationStatement, AstStatementVariant, AstVariableDeclarationStatement,
};

use super::super::instruction::Instruction;
use super::Builder;

impl<'environment> Builder<'environment> {
    pub fn emit_module(&mut self, AstModule { statements, .. }: &AstModule) {
        self.environment.push_scope();
        let statements = self.hoist(statements);
        for statement in statements {
            self.emit_statement(&statement);
        }
        self.environment.pop_scope();
    }

    pub fn emit_block(&mut self, AstBlock { statements, .. }: &AstBlock) {
        self.environment.push_scope();
        let statements = self.hoist(statements);
        for statement in statements {
            self.emit_statement(&statement);
        }
        self.environment.pop_scope();
    }

    pub fn emit_function_block(&mut self, AstBlock { statements, .. }: &AstBlock) {
        let statements = self.hoist(&statements);
        for statement in &statements {
            self.emit_statement(statement);
        }

        if !statements
            .iter()
            .any(|statement| matches!(statement, AstStatementVariant::ReturnStatement(..)))
        {
            self.add(Instruction::PushNull);
        }
    }

    fn hoist<'b>(&mut self, statements: &'b [AstStatementVariant]) -> Vec<&'b AstStatementVariant> {
        let mut result = statements.iter().collect::<Vec<_>>();
        result.sort_by_key(|statement| match statement {
            AstStatementVariant::FunctionDeclarationStatement(..) => 0,
            _ => 1,
        });

        for statement in &result {
            match statement {
                AstStatementVariant::VariableDeclarationStatement(statement) => {
                    self.register_variable_declaration(statement);
                }
                AstStatementVariant::FunctionDeclarationStatement(statement) => {
                    self.register_function_declaration(statement);
                }
                _ => {}
            }
        }

        result
    }

    fn register_function_declaration(
        &mut self,
        AstFunctionDeclarationStatement {
            is_exported,
            function,
            ..
        }: &AstFunctionDeclarationStatement,
    ) {
        if let Some(name) = &function.name {
            if *is_exported {
                self.environment.register_export_variable(name.text.clone());
            } else {
                self.environment.register_local_variable(name.text.clone());
            }
        }
    }

    fn register_variable_declaration(
        &mut self,
        AstVariableDeclarationStatement {
            is_exported, name, ..
        }: &AstVariableDeclarationStatement,
    ) {
        if *is_exported {
            self.environment.register_export_variable(name.text.clone());
        } else {
            self.environment.register_local_variable(name.text.clone());
        }
    }
}
