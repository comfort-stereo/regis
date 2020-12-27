use crate::ast::expression::AstExpressionVariant;
use crate::ast::operator::AssignmentOperator;

use super::super::instruction::Instruction;
use super::Builder;

impl Builder {
    pub fn emit_ncl_operation(&mut self, value: &AstExpressionVariant) {
        self.add(Instruction::Duplicate);
        self.add(Instruction::IsNull);
        let jump_end_if_not_null = self.blank();
        self.add(Instruction::Pop);
        self.emit_expression(value);
        self.set(jump_end_if_not_null, Instruction::JumpUnless(self.end()));
    }

    pub fn emit_and_operation(&mut self, value: &AstExpressionVariant) {
        self.add(Instruction::Duplicate);
        let jump_end_if_false = self.blank();
        self.add(Instruction::Pop);
        self.emit_expression(value);
        self.set(jump_end_if_false, Instruction::JumpUnless(self.end()));
    }

    pub fn emit_or_operation(&mut self, value: &AstExpressionVariant) {
        self.add(Instruction::Duplicate);
        let jump_end_if_true = self.blank();
        self.add(Instruction::Pop);
        self.emit_expression(value);
        self.set(jump_end_if_true, Instruction::JumpIf(self.end()));
    }

    pub fn emit_set_index_value(
        &mut self,
        operator: &AssignmentOperator,
        value: &AstExpressionVariant,
    ) {
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
    }
}
