use super::base::{AstBlock, AstIdentifier};
use super::expression::{AstChainVariant, AstDot, AstExpressionVariant, AstFunction, AstIndex};
use super::grammar::{extract, GrammarPair, GrammarRule, ParseContext};
use super::node::AstNodeInfo;
use super::operator::AssignmentOperator;

#[derive(Debug)]
pub enum AstStatementVariant {
    IfStatement(Box<AstIfStatement>),
    ElseStatement(Box<AstElseStatement>),
    LoopStatement(Box<AstLoopStatement>),
    WhileStatement(Box<AstWhileStatement>),
    ReturnStatement(Box<AstReturnStatement>),
    BreakStatement(Box<AstBreakStatement>),
    ContinueStatement(Box<AstContinueStatement>),
    EchoStatement(Box<AstEchoStatement>),
    FunctionDeclarationStatement(Box<AstFunctionDeclarationStatement>),
    VariableDeclarationStatement(Box<AstVariableDeclarationStatement>),
    VariableAssignmentStatement(Box<AstVariableAssignmentStatement>),
    AstChainAssignmentStatement(AstChainAssignmentStatementVariant),
    AstPushStatement(AstPushStatement),
    AstExpressionStatement(AstExpressionStatement),
}

impl AstStatementVariant {
    pub fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        match pair.as_rule() {
            GrammarRule::if_statement => {
                Self::IfStatement(AstIfStatement::parse(pair, context).into())
            }
            GrammarRule::else_statement => {
                Self::ElseStatement(AstElseStatement::parse(pair, context).into())
            }
            GrammarRule::loop_statement => {
                Self::LoopStatement(AstLoopStatement::parse(pair, context).into())
            }
            GrammarRule::while_statement => {
                Self::WhileStatement(AstWhileStatement::parse(pair, context).into())
            }
            GrammarRule::return_statement => {
                Self::ReturnStatement(AstReturnStatement::parse(pair, context).into())
            }
            GrammarRule::break_statement => {
                Self::BreakStatement(AstBreakStatement::parse(pair, context).into())
            }
            GrammarRule::continue_statement => {
                Self::ContinueStatement(AstContinueStatement::parse(pair, context).into())
            }
            GrammarRule::echo_statement => {
                Self::EchoStatement(AstEchoStatement::parse(pair, context).into())
            }
            GrammarRule::function_declaration_statement => Self::FunctionDeclarationStatement(
                AstFunctionDeclarationStatement::parse(pair, context).into(),
            ),
            GrammarRule::variable_declaration_statement => Self::VariableDeclarationStatement(
                AstVariableDeclarationStatement::parse(pair, context).into(),
            ),
            GrammarRule::variable_assignment_statement => Self::VariableAssignmentStatement(
                AstVariableAssignmentStatement::parse(pair, context).into(),
            ),
            GrammarRule::chain_assignment_statement => Self::AstChainAssignmentStatement(
                AstChainAssignmentStatementVariant::parse(pair, context),
            ),
            GrammarRule::push_statement => {
                Self::AstPushStatement(AstPushStatement::parse(pair, context))
            }
            GrammarRule::expression_statement => {
                Self::AstExpressionStatement(AstExpressionStatement::parse(pair, context))
            }
            _ => unreachable!(),
        }
    }
}
#[derive(Debug)]
pub struct AstIfStatement {
    pub info: AstNodeInfo,
    pub condition: Box<AstExpressionVariant>,
    pub block: Box<AstBlock>,
    pub else_statement: Option<Box<AstElseStatement>>,
}

impl AstIfStatement {
    pub fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::if_statement);
        let (info, mut inner) = extract(pair);
        Self {
            info,
            condition: AstExpressionVariant::parse(inner.next().unwrap(), context).into(),
            block: AstBlock::parse(inner.next().unwrap(), context).into(),
            else_statement: inner
                .next()
                .map(|next| AstElseStatement::parse(next, context).into()),
        }
    }
}

#[derive(Debug)]
pub struct AstElseStatement {
    pub info: AstNodeInfo,
    pub next: Option<AstElseStatementNextVariant>,
}

#[derive(Debug)]
pub enum AstElseStatementNextVariant {
    IfStatement(Box<AstIfStatement>),
    Block(Box<AstBlock>),
}

impl AstElseStatementNextVariant {
    pub fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        match pair.as_rule() {
            GrammarRule::if_statement => {
                Self::IfStatement(AstIfStatement::parse(pair, context).into())
            }
            GrammarRule::block => Self::Block(AstBlock::parse(pair, context).into()),
            _ => unreachable!(),
        }
    }
}

impl AstElseStatement {
    pub fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::else_statement);
        let (info, mut inner) = extract(pair);
        Self {
            info,
            next: inner
                .next()
                .map(|next| AstElseStatementNextVariant::parse(next, context)),
        }
    }
}

#[derive(Debug)]
pub struct AstLoopStatement {
    pub info: AstNodeInfo,
    pub block: Box<AstBlock>,
}

impl AstLoopStatement {
    pub fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::loop_statement);
        let (info, mut inner) = extract(pair);
        Self {
            info,
            block: AstBlock::parse(inner.next().unwrap(), context).into(),
        }
    }
}

#[derive(Debug)]
pub struct AstWhileStatement {
    pub info: AstNodeInfo,
    pub condition: AstExpressionVariant,
    pub block: Box<AstBlock>,
}

impl AstWhileStatement {
    pub fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::while_statement);
        let (info, mut inner) = extract(pair);
        Self {
            info,
            condition: AstExpressionVariant::parse(inner.next().unwrap(), context),
            block: AstBlock::parse(inner.next().unwrap(), context).into(),
        }
    }
}

#[derive(Debug)]
pub struct AstReturnStatement {
    pub info: AstNodeInfo,
    pub value: Option<AstExpressionVariant>,
}

impl AstReturnStatement {
    pub fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::return_statement);
        let (info, mut inner) = extract(pair);
        Self {
            info,
            value: inner
                .next()
                .map(|value| AstExpressionVariant::parse(value, context)),
        }
    }
}

#[derive(Debug)]
pub struct AstBreakStatement {
    pub info: AstNodeInfo,
}

impl AstBreakStatement {
    pub fn parse(pair: GrammarPair, _: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::break_statement);
        Self {
            info: AstNodeInfo::new(&pair),
        }
    }
}

#[derive(Debug)]
pub struct AstContinueStatement {
    pub info: AstNodeInfo,
}

impl AstContinueStatement {
    pub fn parse(pair: GrammarPair, _: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::continue_statement);
        Self {
            info: AstNodeInfo::new(&pair),
        }
    }
}

#[derive(Debug)]
pub struct AstEchoStatement {
    pub info: AstNodeInfo,
    pub value: AstExpressionVariant,
}

impl AstEchoStatement {
    pub fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::echo_statement);
        let (info, mut inner) = extract(pair);
        Self {
            info,
            value: AstExpressionVariant::parse(inner.next().unwrap(), context),
        }
    }
}
#[derive(Debug)]
pub struct AstFunctionDeclarationStatement {
    pub info: AstNodeInfo,
    pub is_exported: bool,
    pub function: Box<AstFunction>,
}

impl AstFunctionDeclarationStatement {
    pub fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::function_declaration_statement);
        let (info, mut inner) = extract(pair);

        let first = inner.next().unwrap();
        let (is_exported, function) = if first.as_rule() == GrammarRule::export {
            (true, inner.next().unwrap())
        } else {
            (false, first)
        };

        Self {
            info,
            is_exported,
            function: AstFunction::parse(function, context).into(),
        }
    }
}

#[derive(Debug)]
pub struct AstVariableDeclarationStatement {
    pub info: AstNodeInfo,
    pub is_exported: bool,
    pub name: Box<AstIdentifier>,
    pub value: AstExpressionVariant,
}

impl AstVariableDeclarationStatement {
    pub fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::variable_declaration_statement);
        let (info, mut inner) = extract(pair);

        let first = inner.next().unwrap();
        let (is_exported, name) = if first.as_rule() == GrammarRule::export {
            (true, inner.next().unwrap())
        } else {
            (false, first)
        };

        Self {
            info,
            is_exported,
            name: AstIdentifier::parse(name, context).into(),
            value: AstExpressionVariant::parse(inner.next().unwrap(), context),
        }
    }
}

#[derive(Debug)]
pub struct AstVariableAssignmentStatement {
    pub info: AstNodeInfo,
    pub name: Box<AstIdentifier>,
    pub operator: AssignmentOperator,
    pub value: AstExpressionVariant,
}

impl AstVariableAssignmentStatement {
    pub fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::variable_assignment_statement);
        let (info, mut inner) = extract(pair);
        Self {
            info,
            name: AstIdentifier::parse(inner.next().unwrap(), context).into(),
            operator: AssignmentOperator::from_rule(&inner.next().unwrap().as_rule()),
            value: AstExpressionVariant::parse(inner.next().unwrap(), context),
        }
    }
}

#[derive(Debug)]
pub enum AstChainAssignmentStatementVariant {
    Index(Box<AstIndexAssignmentStatement>),
    Dot(Box<AstDotAssignmentStatement>),
}

impl AstChainAssignmentStatementVariant {
    pub fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::chain_assignment_statement);
        let (info, mut inner) = extract(pair);

        let target = AstChainVariant::parse(inner.next().unwrap(), context);
        let operator = AssignmentOperator::from_rule(&inner.next().unwrap().as_rule());
        let value = AstExpressionVariant::parse(inner.next().unwrap(), context);

        match target {
            AstChainVariant::Index(index) => Self::Index(
                AstIndexAssignmentStatement {
                    info,
                    index,
                    operator,
                    value,
                }
                .into(),
            ),
            AstChainVariant::Dot(dot) => Self::Dot(
                AstDotAssignmentStatement {
                    info,
                    dot,
                    operator,
                    value,
                }
                .into(),
            ),
            AstChainVariant::Call(..) => unimplemented!(),
            AstChainVariant::Expression(..) => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct AstIndexAssignmentStatement {
    pub info: AstNodeInfo,
    pub index: Box<AstIndex>,
    pub operator: AssignmentOperator,
    pub value: AstExpressionVariant,
}

#[derive(Debug)]
pub struct AstDotAssignmentStatement {
    pub info: AstNodeInfo,
    pub dot: Box<AstDot>,
    pub operator: AssignmentOperator,
    pub value: AstExpressionVariant,
}

#[derive(Debug)]
pub struct AstPushStatement {
    pub info: AstNodeInfo,
    pub target: AstExpressionVariant,
    pub value: AstExpressionVariant,
}

impl AstPushStatement {
    pub fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        let (info, mut inner) = extract(pair);
        AstPushStatement {
            info,
            target: AstExpressionVariant::parse(inner.next().unwrap(), context),
            value: AstExpressionVariant::parse(inner.next().unwrap(), context),
        }
    }
}

#[derive(Debug)]
pub struct AstExpressionStatement {
    pub info: AstNodeInfo,
    pub expression: AstExpressionVariant,
}

impl AstExpressionStatement {
    pub fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        let (info, mut inner) = extract(pair);
        AstExpressionStatement {
            info,
            expression: AstExpressionVariant::parse(inner.next().unwrap(), context),
        }
    }
}
