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

    pub fn emit_null_expr(&mut self, _: &NullExpr) {
        self.add(Instruction::PushNull);
    }

    pub fn emit_boolean_expr(&mut self, BooleanExpr { value, .. }: &BooleanExpr) {
        self.add(Instruction::PushBoolean(*value));
    }

    pub fn emit_int_expr(&mut self, IntExpr { value, .. }: &IntExpr) {
        self.add(Instruction::PushInt(*value));
    }

    pub fn emit_float_expr(&mut self, FloatExpr { value, .. }: &FloatExpr) {
        self.add(Instruction::PushFloat(*value));
    }

    pub fn emit_string_expr(&mut self, StringExpr { value, .. }: &StringExpr) {
        self.add(Instruction::PushString(value.clone()));
    }

    pub fn emit_variable_expr(&mut self, VariableExpr { name, .. }: &VariableExpr) {
        self.emit_variable_push_instruction(&name.text);
    }

    pub fn emit_list_expr(&mut self, ListExpr { values, .. }: &ListExpr) {
        for value in values.iter().rev() {
            self.emit_expr(value);
        }

        self.add(Instruction::CreateList(values.len()));
    }

    pub fn emit_object_expr(&mut self, ObjectExpr { pairs, .. }: &ObjectExpr) {
        for ObjectExprPair { key, value, .. } in pairs.iter().rev() {
            match key {
                ObjectExprKeyVariant::Identifier(Ident { text, .. }) => {
                    self.add(Instruction::PushString(text.clone()))
                }
                ObjectExprKeyVariant::String(StringExpr { value, .. }) => {
                    self.add(Instruction::PushString(value.clone()))
                }
                ObjectExprKeyVariant::Expr(ObjectExprKeyExpr { value, .. }) => {
                    self.emit_expr(value)
                }
            }

            self.emit_expr(value);
        }

        self.add(Instruction::CreateObject(pairs.len()));
    }

    pub fn emit_function_expr(
        &mut self,
        FunctionExpr {
            name,
            parameters,
            body,
            ..
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
        self.add(Instruction::CreateFunction(procedure.into()));
    }

    pub fn emit_wrapped_expr(&mut self, WrappedExpr { value, .. }: &WrappedExpr) {
        self.emit_expr(value)
    }

    pub fn emit_index_expr(&mut self, IndexExpr { target, index, .. }: &IndexExpr) {
        self.emit_expr(target);
        self.emit_expr(index);
        self.add(Instruction::GetIndex);
    }

    pub fn emit_dot_expr(
        &mut self,
        DotExpr {
            target, property, ..
        }: &DotExpr,
    ) {
        self.emit_expr(target);
        self.add(Instruction::PushString(property.text.clone()));
        self.add(Instruction::GetIndex);
    }

    pub fn emit_call_expr(
        &mut self,
        CallExpr {
            target, arguments, ..
        }: &CallExpr,
    ) {
        for argument in arguments.iter() {
            self.emit_expr(argument);
        }

        self.emit_expr(target);
        self.add(Instruction::Call(arguments.len()));
    }

    pub fn emit_unary_operation_expr(
        &mut self,
        UnaryOperationExpr {
            operator, right, ..
        }: &UnaryOperationExpr,
    ) {
        self.emit_expr(right);
        self.add(match operator {
            UnaryOperator::Neg => Instruction::UnaryNeg,
            UnaryOperator::BitNot => Instruction::UnaryBitNot,
            UnaryOperator::Not => Instruction::UnaryNot,
        });
    }

    pub fn emit_binary_operation_expr(
        &mut self,
        BinaryOperationExpr {
            left,
            operator,
            right,
            ..
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
            self.add(eager);

            return;
        }

        self.emit_expr(left);
        match operator {
            BinaryOperator::Ncl => self.emit_ncl_operation(right),
            BinaryOperator::And => self.emit_and_operation(right),
            BinaryOperator::Or => self.emit_or_operation(right),
            _ => unreachable!(),
        };
    }
}
