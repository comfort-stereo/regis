use crate::ast::expression::{
    AstBinaryOperation, AstBoolean, AstCall, AstChainVariant, AstDict, AstDot,
    AstExpressionVariant, AstFunction, AstIdentifier, AstIndex, AstKeyExpression, AstKeyVariant,
    AstLambda, AstLambdaBodyVariant, AstList, AstNull, AstNumber, AstString, AstWrapped,
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

    pub fn emit_boolean(&mut self, boolean: &AstBoolean) {
        self.add(Instruction::PushBoolean(boolean.value));
    }

    pub fn emit_number(&mut self, number: &AstNumber) {
        self.add(Instruction::PushNumber(number.value));
    }

    pub fn emit_string(&mut self, string: &AstString) {
        self.add(Instruction::PushString(string.value.clone()));
    }

    pub fn emit_identifier(&mut self, identifier: &AstIdentifier) {
        self.add(Instruction::PushVariable(identifier.name.clone()));
    }

    pub fn emit_list(&mut self, list: &AstList) {
        for value in list.values.iter().rev() {
            self.emit_expression(value);
        }

        self.add(Instruction::CreateList(list.values.len()));
    }

    pub fn emit_dict(&mut self, dict: &AstDict) {
        for pair in dict.pairs.iter().rev() {
            use AstKeyVariant::*;
            match &pair.key {
                Identifier(AstIdentifier { name, .. }) => {
                    self.add(Instruction::PushString(name.clone()))
                }
                String(AstString { value, .. }) => self.add(Instruction::PushString(value.clone())),
                KeyExpression(AstKeyExpression { value, .. }) => self.emit_expression(value),
            }

            self.emit_expression(&pair.value);
        }

        self.add(Instruction::CreateDict(dict.pairs.len()));
    }

    pub fn emit_function(&mut self, function: &AstFunction) {
        let mut compiler = Builder::new();
        compiler.emit_unscoped_block(&function.block);
        let bytecode = compiler.build();
        let instance = Function::new(
            Some(function.name.name.clone()),
            function
                .parameters
                .iter()
                .map(|parameter| parameter.name.clone())
                .collect(),
            bytecode.into(),
        );

        self.add(Instruction::CreateFunction(instance.into()));
    }

    pub fn emit_lambda(&mut self, lambda: &AstLambda) {
        let mut compiler = Builder::new();
        {
            use AstLambdaBodyVariant::*;
            match &lambda.body {
                Block(block) => compiler.emit_unscoped_block(&block),
                Expression(expression) => compiler.emit_expression(&expression),
            }
        }

        let bytecode = compiler.build();
        let instance = Function::new(
            None,
            lambda
                .parameters
                .iter()
                .map(|parameter| parameter.name.clone())
                .collect(),
            bytecode.into(),
        );

        self.add(Instruction::CreateFunction(instance.into()));
    }

    pub fn emit_wrapped(&mut self, wrapped: &AstWrapped) {
        self.emit_expression(&wrapped.value)
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

    pub fn emit_index(&mut self, index: &AstIndex) {
        self.emit_chain(&index.target);
        self.emit_expression(&index.index);
        self.add(Instruction::GetIndex);
    }

    pub fn emit_dot(&mut self, dot: &AstDot) {
        self.emit_chain(&dot.target);
        self.add(Instruction::PushString(dot.property.name.clone()));
        self.add(Instruction::GetIndex);
    }

    pub fn emit_call(&mut self, call: &AstCall) {
        for argument in call.arguments.iter().rev() {
            self.emit_expression(argument);
        }

        self.emit_chain(&call.target);
        self.add(Instruction::Call(call.arguments.len()));
    }

    pub fn emit_binary_operation(&mut self, binary_operation: &AstBinaryOperation) {
        use BinaryOperator::*;

        if let Some(eager) = match binary_operation.operator {
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
            self.emit_expression(&binary_operation.left);
            self.emit_expression(&binary_operation.right);
            self.add(eager);

            return;
        }

        self.emit_expression(&binary_operation.left);
        match binary_operation.operator {
            Ncl => self.emit_ncl_operation(&binary_operation.right),
            And => self.emit_and_operation(&binary_operation.right),
            Or => self.emit_or_operation(&binary_operation.right),
            _ => unreachable!(),
        };
    }
}
