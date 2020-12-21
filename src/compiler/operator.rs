use crate::ast::expression::AstExpressionVariant;
use crate::ast::operator::AssignmentOperator;

use super::builder::Builder;
use super::bytecode::Instruction;

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
}
