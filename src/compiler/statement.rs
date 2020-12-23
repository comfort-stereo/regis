use crate::ast::operator::AssignmentOperator;
use crate::ast::statement::{
    AstBreakStatement, AstChainAssignmentStatementVariant, AstContinueStatement,
    AstDotAssignmentStatement, AstEchoStatement, AstElseStatement, AstElseStatementNextVariant,
    AstExpressionStatement, AstFunctionStatement, AstIfStatement, AstIndexAssignmentStatement,
    AstLoopStatement, AstReturnStatement, AstStatementVariant, AstVariableAssignmentStatement,
    AstVariableDeclarationStatement, AstWhileStatement,
};

use super::builder::Builder;
use super::bytecode::{Instruction, Marker};

impl Builder {
    pub fn emit_statement(&mut self, variant: &AstStatementVariant) {
        use AstStatementVariant::*;
        match variant {
            IfStatement(if_statement) => self.emit_if_statement(if_statement),
            ElseStatement(else_statement) => self.emit_else_statement(else_statement),
            LoopStatement(loop_statement) => self.emit_loop_statement(loop_statement),
            WhileStatement(while_statement) => self.emit_while_statement(while_statement),
            ReturnStatement(return_statement) => self.emit_return_statement(return_statement),
            BreakStatement(break_statement) => self.emit_break_statement(break_statement),
            ContinueStatement(continue_statement) => {
                self.emit_continue_statement(continue_statement)
            }
            EchoStatement(echo_statement) => self.emit_echo_statement(echo_statement),
            FunctionStatement(function_statement) => {
                self.emit_function_statement(function_statement)
            }
            VariableDeclarationStatement(variable_declaration_statement) => {
                self.emit_variable_declaration_statement(variable_declaration_statement)
            }
            VariableAssignmentStatement(variable_assignment_statement) => {
                self.emit_variable_assignment_statement(variable_assignment_statement)
            }
            AstChainAssignmentStatement(chain_assignment_statement) => {
                self.emit_chain_assignment_statement(chain_assignment_statement)
            }
            ExpressionStatement(expression_statement) => {
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
        use AstElseStatementNextVariant::*;
        match next {
            Some(IfStatement(if_statement)) => self.emit_if_statement(&if_statement),
            Some(Block(block)) => self.emit_block(block),
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
        let start = self.end();
        self.emit_expression(condition);
        self.add(Instruction::JumpIf(self.end() + 2));

        self.blank();
        let jump = self.last();
        self.emit_block(block);
        self.add(Instruction::Jump(start));

        let end = self.end();
        self.mark(end, Marker::LoopEnd);
        self.set(jump, Instruction::Jump(end));
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
        self.add(Instruction::DeclareVariable(name.clone()));
        self.emit_function(function);
        self.add(Instruction::AssignVariable(name.clone()));
    }

    pub fn emit_variable_declaration_statement(
        &mut self,
        AstVariableDeclarationStatement {
            identifier, value, ..
        }: &AstVariableDeclarationStatement,
    ) {
        let name = &identifier.name;
        self.add(Instruction::DeclareVariable(name.clone()));
        self.emit_expression(value);
        self.add(Instruction::AssignVariable(name.clone()));
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
            self.add(Instruction::PushVariable(name.clone()));
        }

        {
            use AssignmentOperator::*;
            match operator {
                Direct => self.emit_expression(value),
                Mul => {
                    self.emit_expression(value);
                    self.add(Instruction::BinaryMul);
                }
                Div => {
                    self.emit_expression(value);
                    self.add(Instruction::BinaryDiv);
                }
                Add => {
                    self.emit_expression(value);
                    self.add(Instruction::BinaryAdd);
                }
                Sub => {
                    self.emit_expression(value);
                    self.add(Instruction::BinarySub);
                }
                And => self.emit_and_operation(value),
                Or => self.emit_or_operation(value),
                Ncl => self.emit_ncl_operation(value),
            }
        }

        self.add(Instruction::AssignVariable(name.clone()));
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
        self.emit_index(index);

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
