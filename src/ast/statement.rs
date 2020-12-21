use super::base::AstBlock;
use super::expression::{
    AstChainVariant, AstDot, AstExpressionVariant, AstFunction, AstIdentifier, AstIndex,
};
use super::node::AstNodeInfo;
use super::operator::AssignmentOperator;
use super::parser::{inner, ParseContext, ParsePair, ParseRule};

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
    FunctionStatement(Box<AstFunctionStatement>),
    VariableDeclarationStatement(Box<AstVariableDeclarationStatement>),
    VariableAssignmentStatement(Box<AstVariableAssignmentStatement>),
    AstChainAssignmentStatement(AstChainAssignmentStatementVariant),
    ExpressionStatement(Box<AstExpressionStatement>),
}

impl AstStatementVariant {
    pub fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        match pair.as_rule() {
            ParseRule::if_statement => {
                Self::IfStatement(AstIfStatement::parse(pair, context).into())
            }
            ParseRule::else_statement => {
                Self::ElseStatement(AstElseStatement::parse(pair, context).into())
            }
            ParseRule::loop_statement => {
                Self::LoopStatement(AstLoopStatement::parse(pair, context).into())
            }
            ParseRule::while_statement => {
                Self::WhileStatement(AstWhileStatement::parse(pair, context).into())
            }
            ParseRule::return_statement => {
                Self::ReturnStatement(AstReturnStatement::parse(pair, context).into())
            }
            ParseRule::break_statement => {
                Self::BreakStatement(AstBreakStatement::parse(pair, context).into())
            }
            ParseRule::continue_statement => {
                Self::ContinueStatement(AstContinueStatement::parse(pair, context).into())
            }
            ParseRule::echo_statement => {
                Self::EchoStatement(AstEchoStatement::parse(pair, context).into())
            }
            ParseRule::function_statement => {
                Self::FunctionStatement(AstFunctionStatement::parse(pair, context).into())
            }
            ParseRule::variable_declaration_statement => Self::VariableDeclarationStatement(
                AstVariableDeclarationStatement::parse(pair, context).into(),
            ),
            ParseRule::variable_assignment_statement => Self::VariableAssignmentStatement(
                AstVariableAssignmentStatement::parse(pair, context).into(),
            ),
            ParseRule::chain_assignment_statement => Self::AstChainAssignmentStatement(
                AstChainAssignmentStatementVariant::parse(pair, context).into(),
            ),
            ParseRule::expression_statement => {
                Self::ExpressionStatement(AstExpressionStatement::parse(pair, context).into())
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
    pub fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::if_statement);
        let (info, mut inner) = inner(pair);
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
    ElseStatement(Box<AstElseStatement>),
}

impl AstElseStatementNextVariant {
    pub fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::else_statement);
        match pair.as_rule() {
            ParseRule::if_statement => {
                Self::IfStatement(AstIfStatement::parse(pair, context).into())
            }
            ParseRule::else_statement => {
                Self::ElseStatement(AstElseStatement::parse(pair, context).into())
            }
            _ => unreachable!(),
        }
    }
}

impl AstElseStatement {
    pub fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        let (info, mut inner) = inner(pair);
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
    pub fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::loop_statement);
        let (info, mut inner) = inner(pair);
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
    pub fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::while_statement);
        let (info, mut inner) = inner(pair);
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
    pub fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::return_statement);
        let (info, mut inner) = inner(pair);
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
    pub fn parse(pair: ParsePair, _: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::break_statement);
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
    pub fn parse(pair: ParsePair, _: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::continue_statement);
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
    pub fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::echo_statement);
        let (info, mut inner) = inner(pair);
        Self {
            info,
            value: AstExpressionVariant::parse(inner.next().unwrap(), context),
        }
    }
}
#[derive(Debug)]
pub struct AstFunctionStatement {
    pub info: AstNodeInfo,
    pub function: Box<AstFunction>,
}

impl AstFunctionStatement {
    pub fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::function_statement);
        let (info, mut inner) = inner(pair);
        Self {
            info,
            function: AstFunction::parse(inner.next().unwrap(), context).into(),
        }
    }
}

#[derive(Debug)]
pub struct AstVariableDeclarationStatement {
    pub info: AstNodeInfo,
    pub identifier: Box<AstIdentifier>,
    pub value: AstExpressionVariant,
}

impl AstVariableDeclarationStatement {
    pub fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::variable_declaration_statement);
        let (info, mut inner) = inner(pair);
        Self {
            info,
            identifier: AstIdentifier::parse(inner.next().unwrap(), context).into(),
            value: AstExpressionVariant::parse(inner.next().unwrap(), context),
        }
    }
}

#[derive(Debug)]
pub struct AstVariableAssignmentStatement {
    pub info: AstNodeInfo,
    pub identifier: Box<AstIdentifier>,
    pub operator: AssignmentOperator,
    pub value: AstExpressionVariant,
}

impl AstVariableAssignmentStatement {
    pub fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::variable_assignment_statement);
        let (info, mut inner) = inner(pair);
        Self {
            info,
            identifier: AstIdentifier::parse(inner.next().unwrap(), context).into(),
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
    pub fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::chain_assignment_statement);
        let (info, mut inner) = inner(pair);

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
pub struct AstExpressionStatement {
    pub info: AstNodeInfo,
    pub expression: AstExpressionVariant,
}

impl AstExpressionStatement {
    pub fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        let (info, mut inner) = inner(pair);
        Self {
            info,
            expression: AstExpressionVariant::parse(inner.next().unwrap(), context),
        }
    }
}
