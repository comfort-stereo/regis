use crate::shared::SharedImmutable;

use super::base::AstBlock;
use super::grammar::{
    content, extract, GrammarAssoc, GrammarOperator, GrammarPair, GrammarPrecClimber, GrammarRule,
    ParseContext,
};
use super::node::AstNodeInfo;
use super::operator::BinaryOperator;
use super::unescape::unescape;

#[derive(Debug)]
pub enum AstExpressionVariant {
    Null(Box<AstNull>),
    Boolean(Box<AstBoolean>),
    Int(Box<AstInt>),
    Float(Box<AstFloat>),
    String(Box<AstString>),
    Identifier(Box<AstIdentifier>),
    List(Box<AstList>),
    Object(Box<AstObject>),
    Function(Box<AstFunction>),
    Lambda(Box<AstLambda>),
    Wrapped(Box<AstWrapped>),
    Chain(AstChainVariant),
    BinaryOperation(Box<AstBinaryOperation>),
}

impl AstExpressionVariant {
    pub fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        match pair.as_rule() {
            GrammarRule::null => Self::Null(AstNull::parse(pair, context).into()),
            GrammarRule::boolean => Self::Boolean(AstBoolean::parse(pair, context).into()),
            GrammarRule::int => Self::Int(AstInt::parse(pair, context).into()),
            GrammarRule::float => Self::Float(AstFloat::parse(pair, context).into()),
            GrammarRule::string => Self::String(AstString::parse(pair, context).into()),
            GrammarRule::identifier => Self::Identifier(AstIdentifier::parse(pair, context).into()),
            GrammarRule::list => Self::List(AstList::parse(pair, context).into()),
            GrammarRule::object => Self::Object(AstObject::parse(pair, context).into()),
            GrammarRule::function => Self::Function(AstFunction::parse(pair, context).into()),
            GrammarRule::lambda => Self::Lambda(AstLambda::parse(pair, context).into()),
            GrammarRule::wrapped => Self::Wrapped(AstWrapped::parse(pair, context).into()),
            GrammarRule::chain => Self::Chain(AstChainVariant::parse(pair, context).into()),
            GrammarRule::binary_operations => {
                Self::BinaryOperation(AstBinaryOperation::parse(pair, context).into())
            }
            _ => unreachable!(),
        }
    }
}

lazy_static! {
    static ref CLIMBER: GrammarPrecClimber = {
        let op = |rule: GrammarRule| GrammarOperator::new(rule, GrammarAssoc::Left);
        GrammarPrecClimber::new(vec![
            op(GrammarRule::operator_binary_push),
            op(GrammarRule::operator_binary_or),
            op(GrammarRule::operator_binary_and),
            op(GrammarRule::operator_binary_eq) | op(GrammarRule::operator_binary_neq),
            op(GrammarRule::operator_binary_gt)
                | op(GrammarRule::operator_binary_lt)
                | op(GrammarRule::operator_binary_gte)
                | op(GrammarRule::operator_binary_lte),
            op(GrammarRule::operator_binary_add) | op(GrammarRule::operator_binary_sub),
            op(GrammarRule::operator_binary_mul) | op(GrammarRule::operator_binary_div),
            op(GrammarRule::operator_binary_ncl),
        ])
    };
}

#[derive(Debug)]
pub struct AstNull {
    pub info: AstNodeInfo,
}

impl AstNull {
    fn parse(pair: GrammarPair, _: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::null);
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
    fn parse(pair: GrammarPair, _: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::boolean);
        Self {
            info: AstNodeInfo::new(&pair),
            value: content(&pair) == "true",
        }
    }
}

#[derive(Debug)]
pub struct AstInt {
    pub info: AstNodeInfo,
    pub value: i64,
}

impl AstInt {
    fn parse(pair: GrammarPair, _: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::int);
        Self {
            info: AstNodeInfo::new(&pair),
            value: content(&pair).parse::<i64>().unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct AstFloat {
    pub info: AstNodeInfo,
    pub value: f64,
}

impl AstFloat {
    fn parse(pair: GrammarPair, _: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::float);
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
    fn parse(pair: GrammarPair, _: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::string);
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
    pub fn parse(pair: GrammarPair, _: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::identifier);
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
    fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::list);
        let (info, inner) = extract(pair);
        Self {
            info,
            values: inner
                .map(|value| AstExpressionVariant::parse(value, context))
                .collect(),
        }
    }
}

#[derive(Debug)]
pub struct AstObject {
    pub info: AstNodeInfo,
    pub pairs: Vec<AstPair>,
}

impl AstObject {
    fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::object);
        let (info, inner) = extract(pair);
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
    fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::pair);
        let (info, mut inner) = extract(pair);
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
    fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        match pair.as_rule() {
            GrammarRule::identifier => Self::Identifier(AstIdentifier::parse(pair, context)),
            GrammarRule::string => Self::String(AstString::parse(pair, context)),
            GrammarRule::key_expression => {
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
    fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::key_expression);
        let (info, mut inner) = extract(pair);
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
    pub fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::function);
        let (info, mut inner) = extract(pair);
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
    fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        match pair.as_rule() {
            GrammarRule::block => Self::Block(AstBlock::parse(pair, context).into()),
            _ => Self::Expression(AstExpressionVariant::parse(pair, context)),
        }
    }
}

impl AstLambda {
    fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::lambda);
        let (info, mut inner) = extract(pair);
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
    fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::binary_operations);
        let (_, inner) = extract(pair);
        let expression = CLIMBER.climb(
            inner,
            |pair: GrammarPair| AstExpressionVariant::parse(pair, context),
            |left: AstExpressionVariant, pair: GrammarPair, right: AstExpressionVariant| {
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
    pub fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::chain);
        let (_, mut inner) = extract(pair);
        let target =
            Self::Expression(AstExpressionVariant::parse(inner.next().unwrap(), context).into());

        inner.fold(target, |target, current| match current.as_rule() {
            GrammarRule::index => Self::Index(AstIndex::parse(current, target, context).into()),
            GrammarRule::dot => Self::Dot(AstDot::parse(current, target, context).into()),
            GrammarRule::call => Self::Call(AstCall::parse(current, target, context).into()),
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
    fn parse(pair: GrammarPair, target: AstChainVariant, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::index);
        let (info, mut inner) = extract(pair);
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
    fn parse(pair: GrammarPair, target: AstChainVariant, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::dot);
        let (info, mut inner) = extract(pair);
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
    fn parse(pair: GrammarPair, target: AstChainVariant, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::call);
        let (info, mut inner) = extract(pair);
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
    fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::wrapped);
        let (info, mut inner) = extract(pair);
        Self {
            info,
            value: AstExpressionVariant::parse(inner.next().unwrap(), context).into(),
        }
    }
}
