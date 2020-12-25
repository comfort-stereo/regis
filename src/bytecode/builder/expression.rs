use crate::ast::expression::{
    AstBinaryOperation, AstBoolean, AstCall, AstChainVariant, AstDict, AstDot,
    AstExpressionVariant, AstFunction, AstIdentifier, AstIndex, AstKeyExpression, AstKeyVariant,
    AstLambda, AstLambdaBodyVariant, AstList, AstNull, AstNumber, AstPair, AstString, AstWrapped,
};
use crate::ast::operator::BinaryOperator;

use super::super::builder::Builder;
use super::super::instruction::Instruction;
use super::super::procedure::Procedure;

impl Builder {
    pub fn emit_expression(&mut self, expression: &AstExpressionVariant) {
        match expression {
            AstExpressionVariant::Null(null) => self.emit_null(null),
            AstExpressionVariant::Boolean(boolean) => self.emit_boolean(boolean),
            AstExpressionVariant::Number(number) => self.emit_number(number),
            AstExpressionVariant::String(string) => self.emit_string(string),
            AstExpressionVariant::Identifier(identifier) => self.emit_identifier(identifier),
            AstExpressionVariant::List(list) => self.emit_list(list),
            AstExpressionVariant::Dict(dict) => self.emit_dict(dict),
            AstExpressionVariant::Function(function) => self.emit_function(function),
            AstExpressionVariant::Lambda(lambda) => self.emit_lambda(lambda),
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

    pub fn emit_number(&mut self, AstNumber { value, .. }: &AstNumber) {
        self.add(Instruction::PushNumber(*value));
    }

    pub fn emit_string(&mut self, AstString { value, .. }: &AstString) {
        self.add(Instruction::PushString(value.clone()));
    }

    pub fn emit_identifier(&mut self, AstIdentifier { name, .. }: &AstIdentifier) {
        self.add(Instruction::PushVariable(self.get_variable_address(name)));
    }

    pub fn emit_list(&mut self, AstList { values, .. }: &AstList) {
        for value in values.iter().rev() {
            self.emit_expression(value);
        }

        self.add(Instruction::CreateList(values.len()));
    }

    pub fn emit_dict(&mut self, AstDict { pairs, .. }: &AstDict) {
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

        self.add(Instruction::CreateDict(pairs.len()));
    }

    pub fn emit_function(
        &mut self,
        AstFunction {
            name,
            parameters,
            block,
            ..
        }: &AstFunction,
    ) {
        let name = name.name.clone();
        let parameters = parameters
            .iter()
            .map(|parameter| parameter.name.clone())
            .collect::<Vec<_>>();

        let bytecode = {
            let mut builder = Builder::new();
            for parameter in &parameters {
                builder.add_variable(parameter.clone());
            }
            builder.emit_block(&block);
            builder.build()
        };

        let procedure = Procedure::new(Some(name), parameters, bytecode);
        self.add(Instruction::CreateFunction(procedure.into()));
    }

    pub fn emit_lambda(
        &mut self,
        AstLambda {
            body, parameters, ..
        }: &AstLambda,
    ) {
        let parameters = parameters
            .iter()
            .map(|parameter| parameter.name.clone())
            .collect::<Vec<_>>();

        let bytecode = {
            let mut builder = Builder::new();
            for parameter in &parameters {
                builder.add_variable(parameter.clone());
            }
            match body {
                AstLambdaBodyVariant::Block(block) => builder.emit_block(&block),
                AstLambdaBodyVariant::Expression(expression) => {
                    builder.emit_expression(&expression)
                }
            }
            builder.build()
        };

        let procedure = Procedure::new(None, parameters, bytecode);
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
