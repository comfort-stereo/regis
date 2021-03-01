use crate::ast::*;

use super::super::instruction::Instruction;
use super::Builder;

impl<'environment> Builder<'environment> {
    pub fn emit_chunk(&mut self, Chunk { stmts, .. }: &Chunk) {
        self.environment.push_scope();
        let stmts = self.hoist(stmts);
        for stmt in stmts {
            self.emit_stmt(&stmt);
        }
        self.environment.pop_scope();
    }

    pub fn emit_block(&mut self, Block { stmts, .. }: &Block) {
        self.environment.push_scope();
        let stmts = self.hoist(stmts);
        for stmt in stmts {
            self.emit_stmt(&stmt);
        }
        self.environment.pop_scope();
    }

    pub fn emit_function_block(&mut self, Block { info, stmts }: &Block) {
        let stmts = self.hoist(&stmts);
        for stmt in &stmts {
            self.emit_stmt(stmt);
        }

        if stmts.iter().any(|stmt| matches!(stmt, Stmt::Return(..))) {
            return;
        }

        self.add(Instruction::PushNull, info);
    }

    fn hoist<'b>(&mut self, stmts: &'b [Stmt]) -> Vec<&'b Stmt> {
        let mut result = stmts.iter().collect::<Vec<_>>();
        result.sort_by_key(|stmt| match stmt {
            Stmt::FunctionDeclaration(..) => 0,
            _ => 1,
        });

        for stmt in &result {
            match stmt {
                Stmt::VariableDeclaration(stmt) => {
                    self.register_variable_declaration(stmt);
                }
                Stmt::FunctionDeclaration(stmt) => {
                    self.register_function_declaration(stmt);
                }
                _ => {}
            }
        }

        result
    }

    fn register_function_declaration(
        &mut self,
        FunctionDeclarationStmt {
            is_exported,
            function,
            ..
        }: &FunctionDeclarationStmt,
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
        VariableDeclarationStmt {
            is_exported, name, ..
        }: &VariableDeclarationStmt,
    ) {
        if *is_exported {
            self.environment.register_export_variable(name.text.clone());
        } else {
            self.environment.register_local_variable(name.text.clone());
        }
    }
}
