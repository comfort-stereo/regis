use crate::ast::*;

use super::super::instruction::Instruction;
use super::super::procedure::Procedure;
use super::super::variable::Parameter;
use super::Builder;

impl<'environment> Builder<'environment> {
    pub fn emit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Null(expr) => self.emit_null_expr(expr),
            Expr::Boolean(expr) => self.emit_boolean_expr(expr),
            Expr::Int(expr) => self.emit_int_expr(expr),
            Expr::Float(expr) => self.emit_float_expr(expr),
            Expr::String(expr) => self.emit_string_expr(expr),
            Expr::Variable(expr) => self.emit_variable_expr(expr),
            Expr::List(expr) => self.emit_list_expr(expr),
            Expr::Object(expr) => self.emit_object_expr(expr),
            Expr::Function(expr) => self.emit_function_expr(expr),
            Expr::Wrapped(expr) => self.emit_wrapped_expr(expr),
            Expr::Index(expr) => self.emit_index_expr(expr),
            Expr::Dot(expr) => self.emit_dot_expr(expr),
            Expr::Call(expr) => self.emit_call_expr(expr),
            Expr::UnaryOperation(expr) => self.emit_unary_operation_expr(expr),
            Expr::BinaryOperation(expr) => self.emit_binary_operation_expr(expr),
        }
    }

    pub fn emit_null_expr(&mut self, NullExpr { info }: &NullExpr) {
        self.add(Instruction::PushNull, info);
    }

    pub fn emit_boolean_expr(&mut self, BooleanExpr { info, value }: &BooleanExpr) {
        self.add(Instruction::PushBoolean(*value), info);
    }

    pub fn emit_int_expr(&mut self, IntExpr { info, value }: &IntExpr) {
        self.add(Instruction::PushInt(*value), info);
    }

    pub fn emit_float_expr(&mut self, FloatExpr { info, value }: &FloatExpr) {
        self.add(Instruction::PushFloat(*value), info);
    }

    pub fn emit_string_expr(&mut self, StringExpr { info, value }: &StringExpr) {
        self.add(Instruction::PushString(value.clone()), info);
    }

    pub fn emit_variable_expr(&mut self, VariableExpr { info, name }: &VariableExpr) {
        self.emit_variable_push_instruction(&name.text, info);
    }

    pub fn emit_list_expr(&mut self, ListExpr { info, values }: &ListExpr) {
        for value in values.iter().rev() {
            self.emit_expr(value);
        }

        self.add(Instruction::CreateList(values.len()), info);
    }

    pub fn emit_object_expr(&mut self, ObjectExpr { info, pairs }: &ObjectExpr) {
        for ObjectExprPair { key, value, .. } in pairs.iter().rev() {
            match key {
                ObjectExprKeyVariant::Identifier(Ident { info, text }) => {
                    self.add(Instruction::PushString(text.clone()), info)
                }
                ObjectExprKeyVariant::String(StringExpr { info, value }) => {
                    self.add(Instruction::PushString(value.clone()), info)
                }
                ObjectExprKeyVariant::Expr(ObjectExprKeyExpr { value, .. }) => {
                    self.emit_expr(value)
                }
            }

            self.emit_expr(value);
        }

        self.add(Instruction::CreateObject(pairs.len()), info);
    }

    pub fn emit_function_expr(
        &mut self,
        FunctionExpr {
            info,
            name,
            parameters,
            body,
        }: &FunctionExpr,
    ) {
        let name = match name {
            Some(name) => Some(name.text.clone()),
            _ => None,
        };
        let parameters = parameters
            .iter()
            .map(|parameter| parameter.text.clone())
            .collect::<Vec<_>>();

        let mut environment = self.environment.for_function();
        {
            for parameter in &parameters {
                environment.add_parameter(Parameter {
                    name: parameter.clone(),
                });
            }
        }

        let mut builder = Builder::new(&mut environment);
        {
            match body {
                FunctionExprBody::Block(block) => {
                    builder.emit_function_block(&block);
                }
                FunctionExprBody::Expr(expr) => builder.emit_expr(&expr),
            }
        }

        let bytecode = builder.build();
        let procedure = Procedure::new(name, bytecode, environment);
        self.add(Instruction::CreateFunction(procedure.into()), info);
    }

    pub fn emit_wrapped_expr(&mut self, WrappedExpr { value, .. }: &WrappedExpr) {
        self.emit_expr(value)
    }

    pub fn emit_index_expr(
        &mut self,
        IndexExpr {
            info,
            target,
            index,
        }: &IndexExpr,
    ) {
        self.emit_expr(target);
        self.emit_expr(index);
        self.add(Instruction::GetIndex, info);
    }

    pub fn emit_dot_expr(
        &mut self,
        DotExpr {
            info,
            target,
            property,
        }: &DotExpr,
    ) {
        self.emit_expr(target);
        self.add(Instruction::PushString(property.text.clone()), info);
        self.add(Instruction::GetIndex, info);
    }

    pub fn emit_call_expr(
        &mut self,
        CallExpr {
            info,
            target,
            arguments,
        }: &CallExpr,
    ) {
        for argument in arguments.iter() {
            self.emit_expr(argument);
        }

        self.emit_expr(target);
        self.add(Instruction::Call(arguments.len()), info);
    }

    pub fn emit_unary_operation_expr(
        &mut self,
        UnaryOperationExpr {
            info,
            operator,
            right,
        }: &UnaryOperationExpr,
    ) {
        self.emit_expr(right);
        self.add(
            match operator {
                UnaryOperator::Neg => Instruction::UnaryNeg,
                UnaryOperator::BitNot => Instruction::UnaryBitNot,
                UnaryOperator::Not => Instruction::UnaryNot,
            },
            info,
        );
    }

    pub fn emit_binary_operation_expr(
        &mut self,
        BinaryOperationExpr {
            info,
            left,
            operator,
            right,
        }: &BinaryOperationExpr,
    ) {
        if let Some(eager) = match operator {
            BinaryOperator::Mul => Some(Instruction::BinaryMul),
            BinaryOperator::Div => Some(Instruction::BinaryDiv),
            BinaryOperator::Add => Some(Instruction::BinaryAdd),
            BinaryOperator::Sub => Some(Instruction::BinarySub),
            BinaryOperator::Gt => Some(Instruction::BinaryGt),
            BinaryOperator::Lt => Some(Instruction::BinaryLt),
            BinaryOperator::Gte => Some(Instruction::BinaryGte),
            BinaryOperator::Lte => Some(Instruction::BinaryLte),
            BinaryOperator::Eq => Some(Instruction::BinaryEq),
            BinaryOperator::Neq => Some(Instruction::BinaryNeq),
            BinaryOperator::Shl => Some(Instruction::BinaryShl),
            BinaryOperator::Shr => Some(Instruction::BinaryShr),
            BinaryOperator::BitAnd => Some(Instruction::BinaryBitAnd),
            BinaryOperator::BitOr => Some(Instruction::BinaryBitOr),
            BinaryOperator::Ncl | BinaryOperator::And | BinaryOperator::Or => None,
        } {
            self.emit_expr(left);
            self.emit_expr(right);
            self.add(eager, info);

            return;
        }

        self.emit_expr(left);
        match operator {
            BinaryOperator::Ncl => self.emit_ncl_operation(right, info),
            BinaryOperator::And => self.emit_and_operation(right, info),
            BinaryOperator::Or => self.emit_or_operation(right, info),
            _ => unreachable!(),
        };
    }
}
