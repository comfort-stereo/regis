use crate::ast::base::{AstBlock, AstModule};
use crate::ast::statement::AstStatementVariant;
use crate::bytecode::{Variable, VariableVariant};

use super::super::instruction::Instruction;
use super::Builder;

impl<'parent> Builder<'parent> {
    pub fn emit_module(&mut self, AstModule { statements, .. }: &AstModule) {
        self.push_scope();
        let statements = self.hoist(statements);
        for statement in statements {
            self.emit_statement(&statement);
        }
        self.pop_scope();
    }

    pub fn emit_block(&mut self, AstBlock { statements, .. }: &AstBlock) {
        self.push_scope();
        let statements = self.hoist(statements);
        for statement in statements {
            self.emit_statement(&statement);
        }
        self.pop_scope();
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
            AstStatementVariant::FunctionStatement(..) => 0,
            _ => 1,
        });

        for statement in &result {
            match statement {
                AstStatementVariant::VariableDeclarationStatement(
                    variable_declaraion_statement,
                ) => {
                    self.add_variable(Variable {
                        name: variable_declaraion_statement.name.text.clone(),
                        variant: VariableVariant::Local,
                    });
                }
                AstStatementVariant::FunctionStatement(..) => {}
                _ => {}
            }
        }

        result
    }
}
