use super::base::*;
use super::expression::*;
use super::statement::*;

pub enum AstTraverseVariant<'a> {
    // Base
    Module(&'a AstModule),
    Block(&'a AstBlock),
    Identifier(&'a AstIdentifier),
    // Expressions
    Null(&'a AstNull),
    Boolean(&'a AstBoolean),
    Int(&'a AstInt),
    Float(&'a AstFloat),
    String(&'a AstString),
    Variable(&'a AstVariable),
    List(&'a AstList),
    Object(&'a AstObject),
    Function(&'a AstFunction),
    Wrapped(&'a AstWrapped),
    Chain(&'a AstChainVariant),
    BinaryOperation(&'a AstBinaryOperation),
    // Statements
    IfStatement(&'a AstIfStatement),
    ElseStatement(&'a AstElseStatement),
    LoopStatement(&'a AstLoopStatement),
    WhileStatement(&'a AstWhileStatement),
    ReturnStatement(&'a AstReturnStatement),
    BreakStatement(&'a AstBreakStatement),
    ContinueStatement(&'a AstContinueStatement),
    EchoStatement(&'a AstEchoStatement),
    FunctionStatement(&'a AstFunctionStatement),
    VariableDeclarationStatement(&'a AstVariableDeclarationStatement),
    VariableAssignmentStatement(&'a AstVariableAssignmentStatement),
    AstChainAssignmentStatement(&'a AstChainAssignmentStatementVariant),
    ExpressionStatement(&'a AstExpressionStatement),
}

impl<'a> AstTraverseVariant<'a> {
    fn from_expression(expression: &'a AstExpressionVariant) -> Self {
        match expression {
            AstExpressionVariant::Null(expression) => Self::Null(expression),
            AstExpressionVariant::Boolean(expression) => Self::Boolean(expression),
            AstExpressionVariant::Int(expression) => Self::Int(expression),
            AstExpressionVariant::Float(expression) => Self::Float(expression),
            AstExpressionVariant::String(expression) => Self::String(expression),
            AstExpressionVariant::Variable(expression) => Self::Variable(expression),
            AstExpressionVariant::List(expression) => Self::List(expression),
            AstExpressionVariant::Object(expression) => Self::Object(expression),
            AstExpressionVariant::Function(expression) => Self::Function(expression),
            AstExpressionVariant::Wrapped(expression) => Self::Wrapped(expression),
            AstExpressionVariant::Chain(expression) => Self::Chain(expression),
            AstExpressionVariant::BinaryOperation(expression) => Self::BinaryOperation(expression),
        }
    }

    fn from_statement(statement: &'a AstStatementVariant) -> Self {
        match statement {
            AstStatementVariant::IfStatement(statement) => Self::IfStatement(statement),
            AstStatementVariant::ElseStatement(statement) => Self::ElseStatement(statement),
            AstStatementVariant::LoopStatement(statement) => Self::LoopStatement(statement),
            AstStatementVariant::WhileStatement(statement) => Self::WhileStatement(statement),
            AstStatementVariant::ReturnStatement(statement) => Self::ReturnStatement(statement),
            AstStatementVariant::BreakStatement(statement) => Self::BreakStatement(statement),
            AstStatementVariant::ContinueStatement(statement) => Self::ContinueStatement(statement),
            AstStatementVariant::EchoStatement(statement) => Self::EchoStatement(statement),
            AstStatementVariant::FunctionStatement(statement) => Self::FunctionStatement(statement),
            AstStatementVariant::VariableDeclarationStatement(statement) => {
                Self::VariableDeclarationStatement(statement)
            }
            AstStatementVariant::VariableAssignmentStatement(statement) => {
                Self::VariableAssignmentStatement(statement)
            }
            AstStatementVariant::AstChainAssignmentStatement(statement) => {
                Self::AstChainAssignmentStatement(statement)
            }
            AstStatementVariant::ExpressionStatement(statement) => {
                Self::ExpressionStatement(statement)
            }
        }
    }
}

type AstTraverseFilter<'a> = fn(current: &AstTraverseVariant<'a>) -> TraversalState;

#[derive(Debug, PartialEq, Eq)]
enum TraversalState {
    Continue,
    Stop,
    Exit,
}

struct AstTraversal<'a> {
    stack: Vec<AstTraverseVariant<'a>>,
    filter: Option<AstTraverseFilter<'a>>,
}

impl<'a> AstTraversal<'a> {
    pub fn new(root: AstTraverseVariant<'a>, filter: Option<AstTraverseFilter<'a>>) -> Self {
        Self {
            stack: vec![root],
            filter,
        }
    }
}

impl<'a> Iterator for AstTraversal<'a> {
    type Item = AstTraverseVariant<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = match self.stack.pop() {
            Some(current) => current,
            None => return None,
        };

        let state = if let Some(state_function) = self.filter {
            state_function(&current)
        } else {
            TraversalState::Continue
        };

        if state == TraversalState::Exit {
            return None;
        }

        if state == TraversalState::Stop {
            return Some(current);
        }

        match &current {
            // Base
            AstTraverseVariant::Module(AstModule { statements, .. }) => {
                self.stack
                    .extend(statements.iter().map(AstTraverseVariant::from_statement));
            }
            AstTraverseVariant::Block(AstBlock { statements, .. }) => {
                self.stack
                    .extend(statements.iter().map(AstTraverseVariant::from_statement));
            }
            AstTraverseVariant::Identifier(..) => {}
            // Expressions
            AstTraverseVariant::Null(..) => {}
            AstTraverseVariant::Boolean(..) => {}
            AstTraverseVariant::Int(..) => {}
            AstTraverseVariant::Float(..) => {}
            AstTraverseVariant::String(..) => {}
            AstTraverseVariant::Variable(AstVariable { name, .. }) => {
                self.stack.push(AstTraverseVariant::Identifier(name));
            }
            AstTraverseVariant::List(AstList { values, .. }) => {
                self.stack
                    .extend(values.iter().map(AstTraverseVariant::from_expression));
            }
            AstTraverseVariant::Object(AstObject { pairs, .. }) => {
                // TODO: Yield pairs directly.
                for AstPair { key, value, .. } in pairs {
                    match key {
                        AstKeyVariant::Identifier(identifier) => {
                            self.stack.push(AstTraverseVariant::Identifier(identifier));
                        }
                        AstKeyVariant::String(string) => {
                            self.stack.push(AstTraverseVariant::String(string));
                        }
                        AstKeyVariant::KeyExpression(AstKeyExpression { value, .. }) => {
                            self.stack.push(AstTraverseVariant::from_expression(value));
                        }
                    }

                    self.stack.push(AstTraverseVariant::from_expression(value));
                }
            }
            AstTraverseVariant::Function(AstFunction {
                name,
                parameters,
                body,
                ..
            }) => {
                if let Some(name) = name {
                    self.stack.push(AstTraverseVariant::Identifier(&name));
                }
                self.stack.extend(
                    parameters
                        .iter()
                        .map(|parameter| AstTraverseVariant::Identifier(&parameter)),
                );
                self.stack.push(match body {
                    AstFunctionBodyVariant::Block(block) => AstTraverseVariant::Block(block),
                    AstFunctionBodyVariant::Expression(expression) => {
                        AstTraverseVariant::from_expression(expression)
                    }
                });
            }
            AstTraverseVariant::Wrapped(AstWrapped { value, .. }) => {
                self.stack.push(AstTraverseVariant::from_expression(value));
            }
            AstTraverseVariant::Chain(variant) => match variant {
                AstChainVariant::Index(index) => {
                    self.stack.push(AstTraverseVariant::Chain(&index.target));
                    self.stack
                        .push(AstTraverseVariant::from_expression(&index.index));
                }
                AstChainVariant::Dot(dot) => {
                    self.stack.push(AstTraverseVariant::Chain(&dot.target));
                    self.stack
                        .push(AstTraverseVariant::Identifier(&dot.property));
                }
                AstChainVariant::Call(call) => {
                    self.stack.push(AstTraverseVariant::Chain(&call.target));
                    self.stack.extend(
                        call.arguments
                            .iter()
                            .map(|argument| AstTraverseVariant::from_expression(argument)),
                    );
                }
                AstChainVariant::Expression(expression) => {
                    self.stack
                        .push(AstTraverseVariant::from_expression(expression));
                }
            },
            AstTraverseVariant::BinaryOperation(AstBinaryOperation { left, right, .. }) => {
                self.stack.push(AstTraverseVariant::from_expression(left));
                self.stack.push(AstTraverseVariant::from_expression(right));
            }
            AstTraverseVariant::IfStatement(AstIfStatement {
                condition,
                block,
                else_statement,
                ..
            }) => {
                self.stack
                    .push(AstTraverseVariant::from_expression(condition));
                self.stack.push(AstTraverseVariant::Block(block));
                if let Some(else_statement) = else_statement {
                    self.stack
                        .push(AstTraverseVariant::ElseStatement(else_statement))
                }
            }
            AstTraverseVariant::ElseStatement(AstElseStatement { next, .. }) => {
                if let Some(next) = next {
                    self.stack.push(match next {
                        AstElseStatementNextVariant::Block(block) => {
                            AstTraverseVariant::Block(block)
                        }
                        AstElseStatementNextVariant::IfStatement(if_statement) => {
                            AstTraverseVariant::IfStatement(if_statement)
                        }
                    })
                }
            }
            AstTraverseVariant::LoopStatement(AstLoopStatement { block, .. }) => {
                self.stack.push(AstTraverseVariant::Block(block));
            }
            AstTraverseVariant::WhileStatement(AstWhileStatement {
                condition, block, ..
            }) => {
                self.stack
                    .push(AstTraverseVariant::from_expression(condition));
                self.stack.push(AstTraverseVariant::Block(block));
            }
            AstTraverseVariant::ReturnStatement(AstReturnStatement { value, .. }) => {
                if let Some(value) = value {
                    self.stack.push(AstTraverseVariant::from_expression(value));
                }
            }
            AstTraverseVariant::BreakStatement(..) => {}
            AstTraverseVariant::ContinueStatement(..) => {}
            AstTraverseVariant::EchoStatement(AstEchoStatement { value, .. }) => {
                self.stack.push(AstTraverseVariant::from_expression(value));
            }
            AstTraverseVariant::FunctionStatement(AstFunctionStatement { function, .. }) => {
                self.stack.push(AstTraverseVariant::Function(function));
            }
            AstTraverseVariant::VariableDeclarationStatement(AstVariableDeclarationStatement {
                name,
                value,
                ..
            }) => {
                self.stack.push(AstTraverseVariant::Identifier(name));
                self.stack.push(AstTraverseVariant::from_expression(value));
            }
            AstTraverseVariant::VariableAssignmentStatement(AstVariableAssignmentStatement {
                name,
                value,
                ..
            }) => {
                self.stack.push(AstTraverseVariant::Identifier(name));
                self.stack.push(AstTraverseVariant::from_expression(value));
            }
            // TODO: Yield index and dot expressions directly.
            AstTraverseVariant::AstChainAssignmentStatement(variant) => match variant {
                AstChainAssignmentStatementVariant::Index(index) => {
                    self.stack
                        .push(AstTraverseVariant::from_expression(&index.index.index));
                    self.stack
                        .push(AstTraverseVariant::from_expression(&index.value));
                }
                AstChainAssignmentStatementVariant::Dot(dot) => {
                    self.stack
                        .push(AstTraverseVariant::Identifier(&dot.dot.property));
                    self.stack
                        .push(AstTraverseVariant::from_expression(&dot.value));
                }
            },
            AstTraverseVariant::ExpressionStatement(AstExpressionStatement {
                expression, ..
            }) => {
                self.stack
                    .push(AstTraverseVariant::from_expression(expression));
            }
        }

        Some(current)
    }
}
