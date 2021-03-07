use crate::ast::{AssignmentOperator, Expr, NodeInfo};

use super::super::instruction::Instruction;
use super::Builder;

impl<'environment> Builder<'environment> {
    pub fn emit_ncl_operation(&mut self, value: &Expr, origin: &NodeInfo) {
        self.add(Instruction::Duplicate, origin);
        self.add(Instruction::IsNull, origin);
        let jump_end_if_not_null = self.blank(origin);
        self.add(Instruction::Pop, origin);
        self.emit_expr(value);
        self.set(
            jump_end_if_not_null,
            Instruction::JumpUnless(self.end()),
            origin,
        );
    }

    pub fn emit_and_operation(&mut self, value: &Expr, origin: &NodeInfo) {
        self.add(Instruction::Duplicate, origin);
        let jump_end_if_false = self.blank(origin);
        self.add(Instruction::Pop, origin);
        self.emit_expr(value);
        self.set(
            jump_end_if_false,
            Instruction::JumpUnless(self.end()),
            origin,
        );
    }

    pub fn emit_or_operation(&mut self, value: &Expr, origin: &NodeInfo) {
        self.add(Instruction::Duplicate, origin);
        let jump_end_if_true = self.blank(origin);
        self.add(Instruction::Pop, origin);
        self.emit_expr(value);
        self.set(jump_end_if_true, Instruction::JumpIf(self.end()), origin);
    }

    pub fn emit_set_index_value(
        &mut self,
        operator: &AssignmentOperator,
        value: &Expr,
        origin: &NodeInfo,
    ) {
        match operator {
            AssignmentOperator::Assign => self.emit_expr(value),
            AssignmentOperator::MulAssign => {
                self.emit_expr(value);
                self.add(Instruction::BinaryMul, origin);
            }
            AssignmentOperator::DivAssign => {
                self.emit_expr(value);
                self.add(Instruction::BinaryDiv, origin);
            }
            AssignmentOperator::AddAssign => {
                self.emit_expr(value);
                self.add(Instruction::BinaryAdd, origin);
            }
            AssignmentOperator::SubAssign => {
                self.emit_expr(value);
                self.add(Instruction::BinarySub, origin);
            }
            AssignmentOperator::NclAssign => self.emit_ncl_operation(value, origin),
        }
    }
}
