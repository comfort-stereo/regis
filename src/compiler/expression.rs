use crate::ast::expression::{
    AstBinaryOperation, AstBoolean, AstCall, AstChainVariant, AstDict, AstDot,
    AstExpressionVariant, AstFunction, AstIdentifier, AstIndex, AstKeyExpression, AstKeyVariant,
    AstLambda, AstLambdaBodyVariant, AstList, AstNull, AstNumber, AstPair, AstString, AstWrapped,
};
use crate::ast::operator::BinaryOperator;
use crate::function::Function;

use super::builder::Builder;
use super::bytecode::Instruction;

impl Builder {
    pub fn emit_expression(&mut self, expression: &AstExpressionVariant) {
        use AstExpressionVariant::*;
        match expression {
            Null(null) => self.emit_null(null),
            Boolean(boolean) => self.emit_boolean(boolean),
            Number(number) => self.emit_number(number),
            String(string) => self.emit_string(string),
            Identifier(identifier) => self.emit_identifier(identifier),
            List(list) => self.emit_list(list),
            Dict(dict) => self.emit_dict(dict),
            Function(function) => self.emit_function(function),
            Lambda(lambda) => self.emit_lambda(lambda),
            Wrapped(wrapped) => self.emit_wrapped(wrapped),
            Chain(chain) => self.emit_chain(chain),
            BinaryOperation(binary_operation) => self.emit_binary_operation(binary_operation),
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
        self.add(Instruction::PushVariable(name.clone()));
    }

    pub fn emit_list(&mut self, AstList { values, .. }: &AstList) {
        for value in values.iter().rev() {
            self.emit_expression(value);
        }

        self.add(Instruction::CreateList(values.len()));
    }

    pub fn emit_dict(&mut self, AstDict { pairs, .. }: &AstDict) {
        for AstPair { key, value, .. } in pairs.iter().rev() {
            use AstKeyVariant::*;
            match key {
                Identifier(AstIdentifier { name, .. }) => {
                    self.add(Instruction::PushString(name.clone()))
                }
                String(AstString { value, .. }) => self.add(Instruction::PushString(value.clone())),
                KeyExpression(AstKeyExpression { value, .. }) => self.emit_expression(value),
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
        let mut compiler = Builder::new();
        compiler.emit_unscoped_block(&block);
        let bytecode = compiler.build();
        let instance = Function::new(
            Some(name.name.clone()),
            parameters
                .iter()
                .map(|parameter| parameter.name.clone())
                .collect(),
            bytecode.into(),
        );

        self.add(Instruction::CreateFunction(instance.into()));
    }

    pub fn emit_lambda(
        &mut self,
        AstLambda {
            body, parameters, ..
        }: &AstLambda,
    ) {
        let mut compiler = Builder::new();
        {
            use AstLambdaBodyVariant::*;
            match body {
                Block(block) => compiler.emit_unscoped_block(&block),
                Expression(expression) => compiler.emit_expression(&expression),
            }
        }

        let bytecode = compiler.build();
        let instance = Function::new(
            None,
            parameters
                .iter()
                .map(|parameter| parameter.name.clone())
                .collect(),
            bytecode.into(),
        );

        self.add(Instruction::CreateFunction(instance.into()));
    }

    pub fn emit_wrapped(&mut self, AstWrapped { value, .. }: &AstWrapped) {
        self.emit_expression(value)
    }

    pub fn emit_chain(&mut self, chain: &AstChainVariant) {
        use AstChainVariant::*;
        match chain {
            Index(index) => self.emit_index(&index),
            Dot(dot) => self.emit_dot(&dot),
            Call(call) => self.emit_call(&call),
            Expression(expression) => self.emit_expression(&expression),
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
        for argument in arguments.iter().rev() {
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
        use BinaryOperator::*;

        if let Some(eager) = match operator {
            Mul => Some(Instruction::BinaryMul),
            Div => Some(Instruction::BinaryDiv),
            Add => Some(Instruction::BinaryAdd),
            Sub => Some(Instruction::BinarySub),
            Gt => Some(Instruction::BinaryGt),
            Lt => Some(Instruction::BinaryLt),
            Gte => Some(Instruction::BinaryGte),
            Lte => Some(Instruction::BinaryLte),
            Eq => Some(Instruction::BinaryEq),
            Neq => Some(Instruction::BinaryNeq),
            Push => Some(Instruction::BinaryPush),
            Ncl | BinaryOperator::And | BinaryOperator::Or => None,
        } {
            self.emit_expression(left);
            self.emit_expression(right);
            self.add(eager);

            return;
        }

        self.emit_expression(left);
        match operator {
            Ncl => self.emit_ncl_operation(right),
            And => self.emit_and_operation(right),
            Or => self.emit_or_operation(right),
            _ => unreachable!(),
        };
    }
}
