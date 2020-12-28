use crate::ast::expression::{
    AstBinaryOperation, AstBoolean, AstCall, AstChainVariant, AstDot, AstExpressionVariant,
    AstFloat, AstFunction, AstFunctionBodyVariant, AstIdentifier, AstIndex, AstInt,
    AstKeyExpression, AstKeyVariant, AstList, AstNull, AstObject, AstPair, AstString, AstWrapped,
};
use crate::ast::operator::BinaryOperator;

use super::super::instruction::Instruction;
use super::super::procedure::Procedure;
use super::super::variable::Parameter;
use super::Builder;

impl Builder {
    pub fn emit_expression(&mut self, expression: &AstExpressionVariant) {
        match expression {
            AstExpressionVariant::Null(null) => self.emit_null(null),
            AstExpressionVariant::Boolean(boolean) => self.emit_boolean(boolean),
            AstExpressionVariant::Int(int) => self.emit_int(int),
            AstExpressionVariant::Float(float) => self.emit_float(float),
            AstExpressionVariant::String(string) => self.emit_string(string),
            AstExpressionVariant::Identifier(identifier) => self.emit_identifier(identifier),
            AstExpressionVariant::List(list) => self.emit_list(list),
            AstExpressionVariant::Object(object) => self.emit_object(object),
            AstExpressionVariant::Function(function) => self.emit_function(function),
            AstExpressionVariant::Wrapped(wrapped) => self.emit_wrapped(wrapped),
            AstExpressionVariant::Chain(chain) => self.emit_chain(chain),
            AstExpressionVariant::BinaryOperation(binary_operation) => {
                self.emit_binary_operation(binary_operation)
            }
        }
    }

    pub fn emit_null(&mut self, _: &AstNull) {
        self.add(Instruction::PushNull);
    }

    pub fn emit_boolean(&mut self, AstBoolean { value, .. }: &AstBoolean) {
        self.add(Instruction::PushBoolean(*value));
    }

    pub fn emit_int(&mut self, AstInt { value, .. }: &AstInt) {
        self.add(Instruction::PushInt(*value));
    }

    pub fn emit_float(&mut self, AstFloat { value, .. }: &AstFloat) {
        self.add(Instruction::PushFloat(*value));
    }

    pub fn emit_string(&mut self, AstString { value, .. }: &AstString) {
        self.add(Instruction::PushString(value.clone()));
    }

    pub fn emit_identifier(&mut self, AstIdentifier { name, .. }: &AstIdentifier) {
        let address = self
            .environment()
            .borrow_mut()
            .get_or_capture_variable_address(name);
        self.add(Instruction::PushVariable(address));
    }

    pub fn emit_list(&mut self, AstList { values, .. }: &AstList) {
        for value in values.iter().rev() {
            self.emit_expression(value);
        }

        self.add(Instruction::CreateList(values.len()));
    }

    pub fn emit_object(&mut self, AstObject { pairs, .. }: &AstObject) {
        for AstPair { key, value, .. } in pairs.iter().rev() {
            match key {
                AstKeyVariant::Identifier(AstIdentifier { name, .. }) => {
                    self.add(Instruction::PushString(name.clone()))
                }
                AstKeyVariant::String(AstString { value, .. }) => {
                    self.add(Instruction::PushString(value.clone()))
                }
                AstKeyVariant::KeyExpression(AstKeyExpression { value, .. }) => {
                    self.emit_expression(value)
                }
            }

            self.emit_expression(value);
        }

        self.add(Instruction::CreateObject(pairs.len()));
    }

    pub fn emit_function(
        &mut self,
        AstFunction {
            name,
            parameters,
            body,
            ..
        }: &AstFunction,
    ) {
        let name = match name {
            Some(identifier) => Some(identifier.name.clone()),
            _ => None,
        };
        let parameters = parameters
            .iter()
            .map(|parameter| parameter.name.clone())
            .collect::<Vec<_>>();

        let bytecode = {
            let mut builder = Builder::new_with_parent_environment(self.environment().clone());
            for parameter in &parameters {
                builder.environment().borrow_mut().add_parameter(Parameter {
                    name: parameter.clone(),
                });
            }
            match body {
                AstFunctionBodyVariant::Block(block) => {
                    builder.emit_function_block(&block);
                }
                AstFunctionBodyVariant::Expression(expression) => {
                    builder.emit_expression(&expression)
                }
            }
            builder.build()
        };

        let procedure = Procedure::new(name, parameters, bytecode);
        self.add(Instruction::CreateFunction(procedure.into()));
    }

    pub fn emit_wrapped(&mut self, AstWrapped { value, .. }: &AstWrapped) {
        self.emit_expression(value)
    }

    pub fn emit_chain(&mut self, chain: &AstChainVariant) {
        match chain {
            AstChainVariant::Index(index) => self.emit_index(&index),
            AstChainVariant::Dot(dot) => self.emit_dot(&dot),
            AstChainVariant::Call(call) => self.emit_call(&call),
            AstChainVariant::Expression(expression) => self.emit_expression(&expression),
        }
    }

    pub fn emit_index(&mut self, AstIndex { target, index, .. }: &AstIndex) {
        self.emit_chain(target);
        self.emit_expression(index);
        self.add(Instruction::GetIndex);
    }

    pub fn emit_dot(
        &mut self,
        AstDot {
            target, property, ..
        }: &AstDot,
    ) {
        self.emit_chain(target);
        self.add(Instruction::PushString(property.name.clone()));
        self.add(Instruction::GetIndex);
    }

    pub fn emit_call(
        &mut self,
        AstCall {
            target, arguments, ..
        }: &AstCall,
    ) {
        for argument in arguments.iter() {
            self.emit_expression(argument);
        }

        self.emit_chain(target);
        self.add(Instruction::Call(arguments.len()));
    }

    pub fn emit_binary_operation(
        &mut self,
        AstBinaryOperation {
            left,
            operator,
            right,
            ..
        }: &AstBinaryOperation,
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
            BinaryOperator::Push => Some(Instruction::BinaryPush),
            BinaryOperator::Ncl | BinaryOperator::And | BinaryOperator::Or => None,
        } {
            self.emit_expression(left);
            self.emit_expression(right);
            self.add(eager);

            return;
        }

        self.emit_expression(left);
        match operator {
            BinaryOperator::Ncl => self.emit_ncl_operation(right),
            BinaryOperator::And => self.emit_and_operation(right),
            BinaryOperator::Or => self.emit_or_operation(right),
            _ => unreachable!(),
        };
    }
}
