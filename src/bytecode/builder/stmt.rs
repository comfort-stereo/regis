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
            info,
            condition,
            block,
            else_clause: next,
        }: &IfStmt,
    ) {
        self.emit_expr(condition);
        let jump_else_or_end_if_not_true = self.blank(info);
        self.emit_block(block);

        if let Some(next) = next {
            let jump_end = self.blank(info);
            self.set(
                jump_else_or_end_if_not_true,
                Instruction::JumpUnless(self.end()),
                info,
            );
            self.emit_else_clause(&next);
            self.set(jump_end, Instruction::Jump(self.end()), info);
        } else {
            self.set(
                jump_else_or_end_if_not_true,
                Instruction::JumpUnless(self.end()),
                info,
            );
        }
    }

    fn emit_else_clause(&mut self, ElseClause { next, .. }: &ElseClause) {
        match next {
            ElseClauseNextVariant::IfStmt(if_stmt) => self.emit_if_stmt(&if_stmt),
            ElseClauseNextVariant::Block(block) => self.emit_block(block),
        }
    }

    pub fn emit_loop_stmt(&mut self, LoopStmt { info, block }: &LoopStmt) {
        self.mark(self.end(), Marker::LoopStart);
        let start = self.end();
        self.emit_block(block);
        self.add(Instruction::Jump(start), info);
        self.mark(self.end(), Marker::LoopEnd);
    }

    pub fn emit_while_stmt(
        &mut self,
        WhileStmt {
            info,
            condition,
            block,
        }: &WhileStmt,
    ) {
        self.mark(self.end(), Marker::LoopStart);
        let start_line = self.end();
        self.emit_expr(condition);
        self.add(Instruction::JumpIf(self.end() + 2), info);

        self.blank(info);
        let jump_line = self.last();
        self.emit_block(block);
        self.add(Instruction::Jump(start_line), info);

        let end_line = self.end();
        self.mark(end_line, Marker::LoopEnd);
        self.set(jump_line, Instruction::Jump(end_line), info);
    }

    pub fn emit_return_stmt(&mut self, ReturnStmt { info, value }: &ReturnStmt) {
        if let Some(value) = value {
            self.emit_expr(&value);
        } else {
            self.add(Instruction::PushNull, info);
        }

        self.add(Instruction::Return, info);
    }

    pub fn emit_break_stmt(&mut self, BreakStmt { info }: &BreakStmt) {
        self.blank(info);
        self.mark(self.last(), Marker::Break);
    }

    pub fn emit_continue_stmt(&mut self, ContinueStmt { info }: &ContinueStmt) {
        self.blank(info);
        self.mark(self.last(), Marker::Continue);
    }

    pub fn emit_function_declaration_stmt(
        &mut self,
        FunctionDeclarationStmt { info, function, .. }: &FunctionDeclarationStmt,
    ) {
        self.emit_function_expr(function);
        if let Some(name) = &function.name {
            // The variable for the function should be registered already due to hoisting, so we
            // just assign it here.
            self.emit_variable_assign_instruction(&name.text, info);
        } else {
            self.add(Instruction::Pop, info);
        }
    }

    pub fn emit_variable_declaration_stmt(
        &mut self,
        VariableDeclarationStmt {
            info, name, value, ..
        }: &VariableDeclarationStmt,
    ) {
        // The variable should be registered already due to hoisting, so we just assign it here.
        self.emit_expr(&value);
        self.emit_variable_assign_instruction(&name.text, info);
    }

    pub fn emit_variable_assignment_stmt(
        &mut self,
        VariableAssignmentStmt {
            info,
            name,
            operator,
            value,
        }: &VariableAssignmentStmt,
    ) {
        let name = &name.text;
        if *operator != AssignmentOperator::Assign {
            self.emit_variable_push_instruction(name, info);
        }

        match operator {
            AssignmentOperator::Assign => self.emit_expr(value),
            AssignmentOperator::MulAssign => {
                self.emit_expr(value);
                self.add(Instruction::BinaryMul, info);
            }
            AssignmentOperator::DivAssign => {
                self.emit_expr(value);
                self.add(Instruction::BinaryDiv, info);
            }
            AssignmentOperator::AddAssign => {
                self.emit_expr(value);
                self.add(Instruction::BinaryAdd, info);
            }
            AssignmentOperator::SubAssign => {
                self.emit_expr(value);
                self.add(Instruction::BinarySub, info);
            }
            AssignmentOperator::NclAssign => self.emit_ncl_operation(value, info),
        }

        self.emit_variable_assign_instruction(name, info);
    }

    pub fn emit_index_assignment_stmt(
        &mut self,
        IndexAssignmentStmt {
            info,
            index_expr,
            operator,
            value,
        }: &IndexAssignmentStmt,
    ) {
        self.emit_expr(&index_expr.target);
        self.emit_expr(&index_expr.index);

        if *operator != AssignmentOperator::Assign {
            self.add(Instruction::DuplicateTop(2), info);
            self.add(Instruction::GetIndex, info);
        }

        self.emit_set_index_value(operator, value, info);
        self.add(Instruction::SetIndex, info);
    }

    pub fn emit_dot_assignment_stmt(
        &mut self,
        DotAssignmentStmt {
            info,
            dot_expr,
            operator,
            value,
        }: &DotAssignmentStmt,
    ) {
        self.emit_expr(&dot_expr.target);
        self.add(
            Instruction::PushString(dot_expr.property.text.clone()),
            info,
        );

        if *operator != AssignmentOperator::Assign {
            self.add(Instruction::DuplicateTop(2), info);
            self.add(Instruction::GetIndex, info);
        }

        self.emit_set_index_value(operator, value, info);
        self.add(Instruction::SetIndex, info);
    }

    pub fn emit_expr_stmt(&mut self, ExprStmt { info, expr }: &ExprStmt) {
        self.emit_expr(expr);
        self.add(Instruction::Pop, info)
    }
}
