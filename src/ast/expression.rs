use crate::shared::SharedImmutable;

use super::base::AstBlock;
use super::node::AstNodeInfo;
use super::operator::BinaryOperator;
use super::parser::{
    content, inner, ParseAssoc, ParseContext, ParseOperator, ParsePair, ParsePrecClimber, ParseRule,
};
use crate::unescape::unescape;

#[derive(Debug)]
pub enum AstExpressionVariant {
    Null(Box<AstNull>),
    Boolean(Box<AstBoolean>),
    Number(Box<AstNumber>),
    String(Box<AstString>),
    Identifier(Box<AstIdentifier>),
    List(Box<AstList>),
    Dict(Box<AstDict>),
    Function(Box<AstFunction>),
    Lambda(Box<AstLambda>),
    Wrapped(Box<AstWrapped>),
    Chain(AstChainVariant),
    BinaryOperation(Box<AstBinaryOperation>),
}

impl AstExpressionVariant {
    pub fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        match pair.as_rule() {
            ParseRule::null => Self::Null(AstNull::parse(pair, context).into()),
            ParseRule::boolean => Self::Boolean(AstBoolean::parse(pair, context).into()),
            ParseRule::number => Self::Number(AstNumber::parse(pair, context).into()),
            ParseRule::string => Self::String(AstString::parse(pair, context).into()),
            ParseRule::identifier => Self::Identifier(AstIdentifier::parse(pair, context).into()),
            ParseRule::list => Self::List(AstList::parse(pair, context).into()),
            ParseRule::dict => Self::Dict(AstDict::parse(pair, context).into()),
            ParseRule::function => Self::Function(AstFunction::parse(pair, context).into()),
            ParseRule::lambda => Self::Lambda(AstLambda::parse(pair, context).into()),
            ParseRule::wrapped => Self::Wrapped(AstWrapped::parse(pair, context).into()),
            ParseRule::chain => Self::Chain(AstChainVariant::parse(pair, context).into()),
            ParseRule::binary_operations => {
                Self::BinaryOperation(AstBinaryOperation::parse(pair, context).into())
            }
            _ => unreachable!(),
        }
    }
}

lazy_static! {
    static ref CLIMBER: ParsePrecClimber = {
        let op = |rule: ParseRule| ParseOperator::new(rule, ParseAssoc::Left);
        ParsePrecClimber::new(vec![
            op(ParseRule::operator_binary_push),
            op(ParseRule::operator_binary_or),
            op(ParseRule::operator_binary_and),
            op(ParseRule::operator_binary_eq) | op(ParseRule::operator_binary_neq),
            op(ParseRule::operator_binary_gt)
                | op(ParseRule::operator_binary_lt)
                | op(ParseRule::operator_binary_gte)
                | op(ParseRule::operator_binary_lte),
            op(ParseRule::operator_binary_add) | op(ParseRule::operator_binary_sub),
            op(ParseRule::operator_binary_mul) | op(ParseRule::operator_binary_div),
            op(ParseRule::operator_binary_ncl),
        ])
    };
}
#[derive(Debug)]
pub struct AstNull {
    pub info: AstNodeInfo,
}

impl AstNull {
    fn parse(pair: ParsePair, _: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::null);
        Self {
            info: AstNodeInfo::new(&pair),
        }
    }
}

#[derive(Debug)]
pub struct AstBoolean {
    pub info: AstNodeInfo,
    pub value: bool,
}

impl AstBoolean {
    fn parse(pair: ParsePair, _: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::boolean);
        Self {
            info: AstNodeInfo::new(&pair),
            value: content(&pair) == "true",
        }
    }
}

#[derive(Debug)]
pub struct AstNumber {
    pub info: AstNodeInfo,
    pub value: f64,
}

impl AstNumber {
    fn parse(pair: ParsePair, _: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::number);
        Self {
            info: AstNodeInfo::new(&pair),
            value: content(&pair).parse::<f64>().unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct AstString {
    pub info: AstNodeInfo,
    pub value: SharedImmutable<String>,
}

impl AstString {
    fn parse(pair: ParsePair, _: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::string);
        Self {
            info: AstNodeInfo::new(&pair),
            value: unescape(&pair.into_inner().next().unwrap().as_str())
                .unwrap()
                .into(),
        }
    }
}

#[derive(Debug)]
pub struct AstIdentifier {
    pub info: AstNodeInfo,
    pub name: SharedImmutable<String>,
}

impl AstIdentifier {
    pub fn parse(pair: ParsePair, _: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::identifier);
        Self {
            info: AstNodeInfo::new(&pair),
            name: content(&pair).into(),
        }
    }
}

#[derive(Debug)]
pub struct AstList {
    pub info: AstNodeInfo,
    pub values: Vec<AstExpressionVariant>,
}

impl AstList {
    fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::list);
        let (info, inner) = inner(pair);
        Self {
            info,
            values: inner
                .map(|value| AstExpressionVariant::parse(value, context))
                .collect(),
        }
    }
}

#[derive(Debug)]
pub struct AstDict {
    pub info: AstNodeInfo,
    pub pairs: Vec<AstPair>,
}

impl AstDict {
    fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::dict);
        let (info, inner) = inner(pair);
        Self {
            info,
            pairs: inner.map(|value| AstPair::parse(value, context)).collect(),
        }
    }
}

#[derive(Debug)]
pub struct AstPair {
    pub info: AstNodeInfo,
    pub key: AstKeyVariant,
    pub value: Box<AstExpressionVariant>,
}

impl AstPair {
    fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::pair);
        let (info, mut inner) = inner(pair);
        Self {
            info,
            key: AstKeyVariant::parse(inner.next().unwrap(), context),
            value: AstExpressionVariant::parse(inner.next().unwrap(), context).into(),
        }
    }
}
#[derive(Debug)]
pub enum AstKeyVariant {
    Identifier(AstIdentifier),
    String(AstString),
    KeyExpression(AstKeyExpression),
}

impl AstKeyVariant {
    fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        match pair.as_rule() {
            ParseRule::identifier => Self::Identifier(AstIdentifier::parse(pair, context)),
            ParseRule::string => Self::String(AstString::parse(pair, context)),
            ParseRule::key_expression => {
                Self::KeyExpression(AstKeyExpression::parse(pair, context))
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct AstKeyExpression {
    pub info: AstNodeInfo,
    pub value: Box<AstExpressionVariant>,
}

impl AstKeyExpression {
    fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::key_expression);
        let (info, mut inner) = inner(pair);
        Self {
            info,
            value: AstExpressionVariant::parse(inner.next().unwrap(), context).into(),
        }
    }
}

#[derive(Debug)]
pub struct AstFunction {
    pub info: AstNodeInfo,
    pub name: Box<AstIdentifier>,
    pub parameters: Vec<AstIdentifier>,
    pub block: Box<AstBlock>,
}

impl AstFunction {
    pub fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::function);
        let (info, mut inner) = inner(pair);
        Self {
            info,
            name: AstIdentifier::parse(inner.next().unwrap(), context).into(),
            parameters: inner
                .next()
                .unwrap()
                .into_inner()
                .map(|parameter| AstIdentifier::parse(parameter, context))
                .collect(),
            block: AstBlock::parse(inner.next().unwrap(), context).into(),
        }
    }
}

#[derive(Debug)]
pub struct AstLambda {
    pub info: AstNodeInfo,
    pub parameters: Vec<AstIdentifier>,
    pub body: AstLambdaBodyVariant,
}

#[derive(Debug)]
pub enum AstLambdaBodyVariant {
    Block(Box<AstBlock>),
    Expression(AstExpressionVariant),
}

impl AstLambdaBodyVariant {
    fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        match pair.as_rule() {
            ParseRule::block => Self::Block(AstBlock::parse(pair, context).into()),
            _ => Self::Expression(AstExpressionVariant::parse(pair, context)),
        }
    }
}

impl AstLambda {
    fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::lambda);
        let (info, mut inner) = inner(pair);
        Self {
            info,
            parameters: inner
                .next()
                .unwrap()
                .into_inner()
                .map(|parameter| AstIdentifier::parse(parameter, context))
                .collect(),
            body: AstLambdaBodyVariant::parse(inner.next().unwrap(), context),
        }
    }
}

#[derive(Debug)]
pub struct AstBinaryOperation {
    pub info: AstNodeInfo,
    pub left: AstExpressionVariant,
    pub operator: BinaryOperator,
    pub right: AstExpressionVariant,
}

impl AstBinaryOperation {
    fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::binary_operations);
        let (_, inner) = inner(pair);
        let expression = CLIMBER.climb(
            inner,
            |pair: ParsePair| AstExpressionVariant::parse(pair, context),
            |left: AstExpressionVariant, pair: ParsePair, right: AstExpressionVariant| {
                let operator = BinaryOperator::from_rule(&pair.as_rule());
                AstExpressionVariant::BinaryOperation(
                    Self {
                        info: AstNodeInfo::new(&pair),
                        left: left.into(),
                        operator,
                        right: right.into(),
                    }
                    .into(),
                )
            },
        );

        match expression {
            AstExpressionVariant::BinaryOperation(binary_operation) => *binary_operation,
            _ => unreachable!(),
        }
    }
}
#[derive(Debug)]
pub enum AstChainVariant {
    Index(Box<AstIndex>),
    Dot(Box<AstDot>),
    Call(Box<AstCall>),
    Expression(Box<AstExpressionVariant>),
}

impl AstChainVariant {
    pub fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::chain);
        let (_, mut inner) = inner(pair);
        let target =
            Self::Expression(AstExpressionVariant::parse(inner.next().unwrap(), context).into());

        inner.fold(target, |target, current| match current.as_rule() {
            ParseRule::index => Self::Index(AstIndex::parse(current, target, context).into()),
            ParseRule::dot => Self::Dot(AstDot::parse(current, target, context).into()),
            ParseRule::call => Self::Call(AstCall::parse(current, target, context).into()),
            _ => unreachable!(),
        })
    }
}

#[derive(Debug)]
pub struct AstIndex {
    pub info: AstNodeInfo,
    pub target: AstChainVariant,
    pub index: Box<AstExpressionVariant>,
}

impl AstIndex {
    fn parse(pair: ParsePair, target: AstChainVariant, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::index);
        let (info, mut inner) = inner(pair);
        Self {
            info,
            target,
            index: AstExpressionVariant::parse(inner.next().unwrap(), context).into(),
        }
    }
}

#[derive(Debug)]
pub struct AstDot {
    pub info: AstNodeInfo,
    pub target: AstChainVariant,
    pub property: Box<AstIdentifier>,
}

impl AstDot {
    fn parse(pair: ParsePair, target: AstChainVariant, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::dot);
        let (info, mut inner) = inner(pair);
        Self {
            info,
            target,
            property: AstIdentifier::parse(inner.next().unwrap(), context).into(),
        }
    }
}

#[derive(Debug)]
pub struct AstCall {
    pub info: AstNodeInfo,
    pub target: AstChainVariant,
    pub arguments: Vec<AstExpressionVariant>,
}

impl AstCall {
    fn parse(pair: ParsePair, target: AstChainVariant, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::call);
        let (info, mut inner) = inner(pair);
        Self {
            info,
            target,
            arguments: inner
                .next()
                .unwrap()
                .into_inner()
                .map(|argument| AstExpressionVariant::parse(argument, context))
                .collect(),
        }
    }
}

#[derive(Debug)]
pub struct AstWrapped {
    pub info: AstNodeInfo,
    pub value: Box<AstExpressionVariant>,
}

impl AstWrapped {
    fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::wrapped);
        let (info, mut inner) = inner(pair);
        Self {
            info,
            value: AstExpressionVariant::parse(inner.next().unwrap(), context).into(),
        }
    }
}
