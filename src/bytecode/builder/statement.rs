use crate::ast::operator::AssignmentOperator;
use crate::ast::statement::{
    AstBreakStatement, AstChainAssignmentStatementVariant, AstContinueStatement,
    AstDotAssignmentStatement, AstEchoStatement, AstElseStatement, AstElseStatementNextVariant,
    AstExpressionStatement, AstFunctionStatement, AstIfStatement, AstIndexAssignmentStatement,
    AstLoopStatement, AstReturnStatement, AstStatementVariant, AstVariableAssignmentStatement,
    AstVariableDeclarationStatement, AstWhileStatement,
};

use super::super::instruction::Instruction;
use super::marker::Marker;
use super::Builder;

impl Builder {
    pub fn emit_statement(&mut self, variant: &AstStatementVariant) {
        match variant {
            AstStatementVariant::IfStatement(if_statement) => self.emit_if_statement(if_statement),
            AstStatementVariant::ElseStatement(else_statement) => {
                self.emit_else_statement(else_statement)
            }
            AstStatementVariant::LoopStatement(loop_statement) => {
                self.emit_loop_statement(loop_statement)
            }
            AstStatementVariant::WhileStatement(while_statement) => {
                self.emit_while_statement(while_statement)
            }
            AstStatementVariant::ReturnStatement(return_statement) => {
                self.emit_return_statement(return_statement)
            }
            AstStatementVariant::BreakStatement(break_statement) => {
                self.emit_break_statement(break_statement)
            }
            AstStatementVariant::ContinueStatement(continue_statement) => {
                self.emit_continue_statement(continue_statement)
            }
            AstStatementVariant::EchoStatement(echo_statement) => {
                self.emit_echo_statement(echo_statement)
            }
            AstStatementVariant::FunctionStatement(function_statement) => {
                self.emit_function_statement(function_statement)
            }
            AstStatementVariant::VariableDeclarationStatement(variable_declaration_statement) => {
                self.emit_variable_declaration_statement(variable_declaration_statement)
            }
            AstStatementVariant::VariableAssignmentStatement(variable_assignment_statement) => {
                self.emit_variable_assignment_statement(variable_assignment_statement)
            }
            AstStatementVariant::AstChainAssignmentStatement(chain_assignment_statement) => {
                self.emit_chain_assignment_statement(chain_assignment_statement)
            }
            AstStatementVariant::ExpressionStatement(expression_statement) => {
                self.emit_expression_statement(expression_statement)
            }
        }
    }

    pub fn emit_if_statement(
        &mut self,
        AstIfStatement {
            condition,
            block,
            else_statement,
            ..
        }: &AstIfStatement,
    ) {
        self.emit_expression(condition);
        let jump_else_or_end_if_not_true = self.blank();
        self.emit_block(block);

        if let Some(else_statement) = else_statement {
            let jump_end = self.blank();
            self.set(
                jump_else_or_end_if_not_true,
                Instruction::JumpUnless(self.end()),
            );
            self.emit_else_statement(&else_statement);
            self.set(jump_end, Instruction::Jump(self.end()));
        } else {
            self.set(
                jump_else_or_end_if_not_true,
                Instruction::JumpUnless(self.end()),
            );
        }
    }

    pub fn emit_else_statement(&mut self, AstElseStatement { next, .. }: &AstElseStatement) {
        match next {
            Some(AstElseStatementNextVariant::IfStatement(if_statement)) => {
                self.emit_if_statement(&if_statement)
            }
            Some(AstElseStatementNextVariant::Block(block)) => self.emit_block(block),
            None => {}
        }
    }

    pub fn emit_loop_statement(&mut self, AstLoopStatement { block, .. }: &AstLoopStatement) {
        self.mark(self.end(), Marker::LoopStart);
        let start = self.end();
        self.emit_block(block);
        self.add(Instruction::Jump(start));
        self.mark(self.end(), Marker::LoopEnd);
    }

    pub fn emit_while_statement(
        &mut self,
        AstWhileStatement {
            condition, block, ..
        }: &AstWhileStatement,
    ) {
        self.mark(self.end(), Marker::LoopStart);
        let start_line = self.end();
        self.emit_expression(condition);
        self.add(Instruction::JumpIf(self.end() + 2));

        self.blank();
        let jump_line = self.last();
        self.emit_block(block);
        self.add(Instruction::Jump(start_line));

        let end_line = self.end();
        self.mark(end_line, Marker::LoopEnd);
        self.set(jump_line, Instruction::Jump(end_line));
    }

    pub fn emit_return_statement(&mut self, AstReturnStatement { value, .. }: &AstReturnStatement) {
        if let Some(value) = value {
            self.emit_expression(&value);
        } else {
            self.add(Instruction::PushNull);
        }

        self.add(Instruction::Return);
    }

    pub fn emit_break_statement(&mut self, _: &AstBreakStatement) {
        self.blank();
        self.mark(self.last(), Marker::Break);
    }

    pub fn emit_continue_statement(&mut self, _: &AstContinueStatement) {
        self.blank();
        self.mark(self.last(), Marker::Continue);
    }

    pub fn emit_echo_statement(&mut self, AstEchoStatement { value, .. }: &AstEchoStatement) {
        self.emit_expression(value);
        self.add(Instruction::Echo);
    }

    pub fn emit_function_statement(
        &mut self,
        AstFunctionStatement { function, .. }: &AstFunctionStatement,
    ) {
        let name = &function.name.name;
        let address = self.add_variable(name.clone());
        self.emit_function(function);
        self.add(Instruction::AssignVariable(address));
    }

    pub fn emit_variable_declaration_statement(
        &mut self,
        AstVariableDeclarationStatement {
            identifier, value, ..
        }: &AstVariableDeclarationStatement,
    ) {
        let name = &identifier.name;
        let address = self.add_variable(name.clone());
        self.emit_expression(value);
        self.add(Instruction::AssignVariable(address));
    }

    pub fn emit_variable_assignment_statement(
        &mut self,
        AstVariableAssignmentStatement {
            identifier,
            operator,
            value,
            ..
        }: &AstVariableAssignmentStatement,
    ) {
        let name = &identifier.name;
        if *operator != AssignmentOperator::Direct {
            self.add(Instruction::PushVariable(self.get_variable_address(name)));
        }

        match operator {
            AssignmentOperator::Direct => self.emit_expression(value),
            AssignmentOperator::Mul => {
                self.emit_expression(value);
                self.add(Instruction::BinaryMul);
            }
            AssignmentOperator::Div => {
                self.emit_expression(value);
                self.add(Instruction::BinaryDiv);
            }
            AssignmentOperator::Add => {
                self.emit_expression(value);
                self.add(Instruction::BinaryAdd);
            }
            AssignmentOperator::Sub => {
                self.emit_expression(value);
                self.add(Instruction::BinarySub);
            }
            AssignmentOperator::And => self.emit_and_operation(value),
            AssignmentOperator::Or => self.emit_or_operation(value),
            AssignmentOperator::Ncl => self.emit_ncl_operation(value),
        }

        self.add(Instruction::AssignVariable(self.get_variable_address(name)));
    }

    pub fn emit_chain_assignment_statement(
        &mut self,
        variant: &AstChainAssignmentStatementVariant,
    ) {
        use AstChainAssignmentStatementVariant::*;
        match variant {
            Index(index_assignment_statement) => {
                self.emit_index_assignment_statement(index_assignment_statement)
            }
            Dot(dot_assignment_statement) => {
                self.emit_dot_assignment_statement(dot_assignment_statement)
            }
        }
    }

    pub fn emit_index_assignment_statement(
        &mut self,
        AstIndexAssignmentStatement {
            index,
            operator,
            value,
            ..
        }: &AstIndexAssignmentStatement,
    ) {
        self.emit_chain(&index.target);
        self.emit_expression(&index.index);

        if *operator != AssignmentOperator::Direct {
            self.add(Instruction::DuplicateTop(2));
            self.add(Instruction::GetIndex);
        }

        self.emit_set_index_value(operator, value);
        self.add(Instruction::SetIndex);
    }

    pub fn emit_dot_assignment_statement(
        &mut self,
        AstDotAssignmentStatement {
            dot,
            operator,
            value,
            ..
        }: &AstDotAssignmentStatement,
    ) {
        self.emit_chain(&dot.target);
        self.add(Instruction::PushString(dot.property.name.clone()));

        if *operator != AssignmentOperator::Direct {
            self.add(Instruction::DuplicateTop(2));
            self.add(Instruction::GetIndex);
        }

        self.emit_set_index_value(operator, value);
        self.add(Instruction::SetIndex);
    }

    pub fn emit_expression_statement(
        &mut self,
        AstExpressionStatement { expression, .. }: &AstExpressionStatement,
    ) {
        self.emit_expression(expression);
        self.add(Instruction::Pop);
    }
}
