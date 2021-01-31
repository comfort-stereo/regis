use crate::ast::AssignmentOperator;
use crate::ast::Expr;

use super::super::instruction::Instruction;
use super::Builder;

impl<'environment> Builder<'environment> {
    pub fn emit_ncl_operation(&mut self, value: &Expr) {
        self.add(Instruction::Duplicate);
        self.add(Instruction::IsNull);
        let jump_end_if_not_null = self.blank();
        self.add(Instruction::Pop);
        self.emit_expr(value);
        self.set(jump_end_if_not_null, Instruction::JumpUnless(self.end()));
    }

    pub fn emit_and_operation(&mut self, value: &Expr) {
        self.add(Instruction::Duplicate);
        let jump_end_if_false = self.blank();
        self.add(Instruction::Pop);
        self.emit_expr(value);
        self.set(jump_end_if_false, Instruction::JumpUnless(self.end()));
    }

    pub fn emit_or_operation(&mut self, value: &Expr) {
        self.add(Instruction::Duplicate);
        let jump_end_if_true = self.blank();
        self.add(Instruction::Pop);
        self.emit_expr(value);
        self.set(jump_end_if_true, Instruction::JumpIf(self.end()));
    }

    pub fn emit_set_index_value(&mut self, operator: &AssignmentOperator, value: &Expr) {
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
    }
}
