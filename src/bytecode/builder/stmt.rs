use crate::ast::*;

use super::super::instruction::Instruction;
use super::marker::Marker;
use super::Builder;

impl<'environment> Builder<'environment> {
    pub fn emit_stmt(&mut self, variant: &Stmt) {
        match variant {
            Stmt::If(stmt) => self.emit_if_stmt(stmt),
            Stmt::Loop(stmt) => self.emit_loop_stmt(stmt),
            Stmt::While(stmt) => self.emit_while_stmt(stmt),
            Stmt::Return(stmt) => self.emit_return_stmt(stmt),
            Stmt::Break(stmt) => self.emit_break_stmt(stmt),
            Stmt::Continue(stmt) => self.emit_continue_stmt(stmt),
            Stmt::FunctionDeclaration(stmt) => self.emit_function_declaration_stmt(stmt),
            Stmt::VariableDeclaration(stmt) => self.emit_variable_declaration_stmt(stmt),
            Stmt::VariableAssignment(stmt) => self.emit_variable_assignment_stmt(stmt),
            Stmt::IndexAssignment(stmt) => self.emit_index_assignment_stmt(stmt),
            Stmt::DotAssignment(stmt) => self.emit_dot_assignment_stmt(stmt),
            Stmt::Expr(stmt) => self.emit_expr_stmt(stmt),
        }
    }

    pub fn emit_if_stmt(
        &mut self,
        IfStmt {
            condition,
            block,
            else_clause: next,
            ..
        }: &IfStmt,
    ) {
        self.emit_expr(condition);
        let jump_else_or_end_if_not_true = self.blank();
        self.emit_block(block);

        if let Some(next) = next {
            let jump_end = self.blank();
            self.set(
                jump_else_or_end_if_not_true,
                Instruction::JumpUnless(self.end()),
            );
            self.emit_else_clause(&next);
            self.set(jump_end, Instruction::Jump(self.end()));
        } else {
            self.set(
                jump_else_or_end_if_not_true,
                Instruction::JumpUnless(self.end()),
            );
        }
    }

    fn emit_else_clause(&mut self, ElseClause { next, .. }: &ElseClause) {
        match next {
            ElseClauseNextVariant::IfStmt(if_stmt) => self.emit_if_stmt(&if_stmt),
            ElseClauseNextVariant::Block(block) => self.emit_block(block),
        }
    }

    pub fn emit_loop_stmt(&mut self, LoopStmt { block, .. }: &LoopStmt) {
        self.mark(self.end(), Marker::LoopStart);
        let start = self.end();
        self.emit_block(block);
        self.add(Instruction::Jump(start));
        self.mark(self.end(), Marker::LoopEnd);
    }

    pub fn emit_while_stmt(
        &mut self,
        WhileStmt {
            condition, block, ..
        }: &WhileStmt,
    ) {
        self.mark(self.end(), Marker::LoopStart);
        let start_line = self.end();
        self.emit_expr(condition);
        self.add(Instruction::JumpIf(self.end() + 2));

        self.blank();
        let jump_line = self.last();
        self.emit_block(block);
        self.add(Instruction::Jump(start_line));

        let end_line = self.end();
        self.mark(end_line, Marker::LoopEnd);
        self.set(jump_line, Instruction::Jump(end_line));
    }

    pub fn emit_return_stmt(&mut self, ReturnStmt { value, .. }: &ReturnStmt) {
        if let Some(value) = value {
            self.emit_expr(&value);
        } else {
            self.add(Instruction::PushNull);
        }

        self.add(Instruction::Return);
    }

    pub fn emit_break_stmt(&mut self, _: &BreakStmt) {
        self.blank();
        self.mark(self.last(), Marker::Break);
    }

    pub fn emit_continue_stmt(&mut self, _: &ContinueStmt) {
        self.blank();
        self.mark(self.last(), Marker::Continue);
    }

    pub fn emit_function_declaration_stmt(
        &mut self,
        FunctionDeclarationStmt { function, .. }: &FunctionDeclarationStmt,
    ) {
        self.emit_function_expr(function);
        if let Some(name) = &function.name {
            // The variable for the function should be registered already due to hoisting, so we
            // just assign it here.
            self.emit_variable_assign_instruction(&name.text);
        } else {
            self.add(Instruction::Pop);
        }
    }

    pub fn emit_variable_declaration_stmt(
        &mut self,
        VariableDeclarationStmt { name, value, .. }: &VariableDeclarationStmt,
    ) {
        // The variable should be registered already due to hoisting, so we just assign it here.
        self.emit_expr(&value);
        self.emit_variable_assign_instruction(&name.text);
    }

    pub fn emit_variable_assignment_stmt(
        &mut self,
        VariableAssignmentStmt {
            name,
            operator,
            value,
            ..
        }: &VariableAssignmentStmt,
    ) {
        let name = &name.text;
        if *operator != AssignmentOperator::Assign {
            self.emit_variable_push_instruction(name);
        }

        match operator {
            AssignmentOperator::Assign => self.emit_expr(value),
            AssignmentOperator::MulAssign => {
                self.emit_expr(value);
                self.add(Instruction::BinaryMul);
            }
            AssignmentOperator::DivAssign => {
                self.emit_expr(value);
                self.add(Instruction::BinaryDiv);
            }
            AssignmentOperator::AddAssign => {
                self.emit_expr(value);
                self.add(Instruction::BinaryAdd);
            }
            AssignmentOperator::SubAssign => {
                self.emit_expr(value);
                self.add(Instruction::BinarySub);
            }
            AssignmentOperator::NclAssign => self.emit_ncl_operation(value),
        }

        self.emit_variable_assign_instruction(name);
    }

    pub fn emit_index_assignment_stmt(
        &mut self,
        IndexAssignmentStmt {
            index_expr,
            operator,
            value,
            ..
        }: &IndexAssignmentStmt,
    ) {
        self.emit_expr(&index_expr.target);
        self.emit_expr(&index_expr.index);

        if *operator != AssignmentOperator::Assign {
            self.add(Instruction::DuplicateTop(2));
            self.add(Instruction::GetIndex);
        }

        self.emit_set_index_value(operator, value);
        self.add(Instruction::SetIndex);
    }

    pub fn emit_dot_assignment_stmt(
        &mut self,
        DotAssignmentStmt {
            dot_expr,
            operator,
            value,
            ..
        }: &DotAssignmentStmt,
    ) {
        self.emit_expr(&dot_expr.target);
        self.add(Instruction::PushString(dot_expr.property.text.clone()));

        if *operator != AssignmentOperator::Assign {
            self.add(Instruction::DuplicateTop(2));
            self.add(Instruction::GetIndex);
        }

        self.emit_set_index_value(operator, value);
        self.add(Instruction::SetIndex);
    }

    pub fn emit_expr_stmt(&mut self, ExprStmt { expr, .. }: &ExprStmt) {
        self.emit_expr(expr);
        self.add(Instruction::Pop)
    }
}
