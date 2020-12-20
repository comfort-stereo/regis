use lazy_static;

use pest::error::Error;
use pest::iterators::{Pair, Pairs};
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::{Parser, Span};
use std::iter::Iterator;
use uuid::Uuid;

use crate::shared::SharedImmutable;
use crate::unescape::unescape;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct RegisParser;

pub type ParseRule = Rule;

#[derive(Debug)]
pub struct AstNode {
    id: Uuid,
    start: usize,
    end: usize,
    variant: AstNodeVariant,
}

impl AstNode {
    pub fn create(span: &Span, variant: AstNodeVariant) -> Box<Self> {
        Box::new(Self {
            id: Uuid::new_v4(),
            start: span.start(),
            end: span.end(),
            variant,
        })
    }

    pub fn variant(&self) -> &AstNodeVariant {
        &self.variant
    }
}

#[derive(Debug)]
pub enum AstNodeVariant {
    Module {
        statements: Vec<Box<AstNode>>,
    },
    VariableDeclarationStatement {
        name: SharedImmutable<String>,
        value: Box<AstNode>,
    },
    VariableAssignmentStatement {
        name: SharedImmutable<String>,
        operator: AssignmentOperator,
        value: Box<AstNode>,
    },
    IndexAssignmentStatement {
        index: Box<AstNode>,
        operator: AssignmentOperator,
        value: Box<AstNode>,
    },
    Null,
    Boolean {
        value: bool,
    },
    Number {
        value: f64,
    },
    String {
        value: SharedImmutable<String>,
    },
    Identifier {
        name: SharedImmutable<String>,
    },
    List {
        values: Vec<Box<AstNode>>,
    },
    Dict {
        pairs: Vec<Box<AstNode>>,
    },
    Pair {
        key: Box<AstNode>,
        value: Box<AstNode>,
    },
    Function {
        name: SharedImmutable<String>,
        parameters: Vec<SharedImmutable<String>>,
        block: Box<AstNode>,
    },
    Lambda {
        parameters: Vec<SharedImmutable<String>>,
        body: Box<AstNode>,
    },
    FunctionStatement {
        function: Box<AstNode>,
    },
    BinaryOperation {
        left: Box<AstNode>,
        operator: BinaryOperator,
        right: Box<AstNode>,
    },
    Index {
        target: Box<AstNode>,
        index: Box<AstNode>,
    },
    Call {
        target: Box<AstNode>,
        arguments: Vec<Box<AstNode>>,
    },
    Wrapped {
        value: Box<AstNode>,
    },
    IfStatement {
        condition: Box<AstNode>,
        block: Box<AstNode>,
        else_statement: Option<Box<AstNode>>,
    },
    ElseStatement {
        next: Box<AstNode>,
    },
    LoopStatement {
        block: Box<AstNode>,
    },
    WhileStatement {
        condition: Box<AstNode>,
        block: Box<AstNode>,
    },
    ReturnStatement {
        value: Option<Box<AstNode>>,
    },
    BreakStatement,
    ContinueStatement,
    EchoStatement {
        value: Box<AstNode>,
    },
    ExpressionStatement {
        expression: Box<AstNode>,
    },
    Block {
        statements: Vec<Box<AstNode>>,
    },
    FunctionBlock {
        statements: Vec<Box<AstNode>>,
    },
    Unknown,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    Ncl,
    Mul,
    Div,
    Add,
    Sub,
    Gt,
    Lt,
    Gte,
    Lte,
    Eq,
    Neq,
    And,
    Or,
    Push,
}

impl BinaryOperator {
    fn from_rule(rule: &Rule) -> Self {
        match rule {
            Rule::operator_binary_ncl => BinaryOperator::Ncl,
            Rule::operator_binary_mul => BinaryOperator::Mul,
            Rule::operator_binary_div => BinaryOperator::Div,
            Rule::operator_binary_add => BinaryOperator::Add,
            Rule::operator_binary_sub => BinaryOperator::Sub,
            Rule::operator_binary_gt => BinaryOperator::Gt,
            Rule::operator_binary_lt => BinaryOperator::Lt,
            Rule::operator_binary_gte => BinaryOperator::Gte,
            Rule::operator_binary_lte => BinaryOperator::Lte,
            Rule::operator_binary_eq => BinaryOperator::Eq,
            Rule::operator_binary_neq => BinaryOperator::Neq,
            Rule::operator_binary_and => BinaryOperator::And,
            Rule::operator_binary_or => BinaryOperator::Or,
            Rule::operator_binary_push => BinaryOperator::Push,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AssignmentOperator {
    Direct,
    Mul,
    Div,
    Add,
    Sub,
    And,
    Or,
    Ncl,
}

impl AssignmentOperator {
    fn from_rule(rule: &Rule) -> Self {
        match rule {
            Rule::operator_assign_direct => AssignmentOperator::Direct,
            Rule::operator_assign_ncl => AssignmentOperator::Ncl,
            Rule::operator_assign_mul => AssignmentOperator::Mul,
            Rule::operator_assign_div => AssignmentOperator::Div,
            Rule::operator_assign_add => AssignmentOperator::Add,
            Rule::operator_assign_sub => AssignmentOperator::Sub,
            Rule::operator_assign_and => AssignmentOperator::And,
            Rule::operator_assign_or => AssignmentOperator::Or,
            _ => unreachable!(),
        }
    }
}

pub fn parse(code: &str) -> Result<Box<AstNode>, Error<Rule>> {
    Ok(build(read(code)?))
}

fn read(code: &str) -> Result<Pair<Rule>, Error<Rule>> {
    let pairs = RegisParser::parse(Rule::module, code)?
        .into_iter()
        .collect::<Vec<_>>();
    for pair in pairs {
        return Ok(pair);
    }

    unreachable!();
}

lazy_static! {
    static ref CLIMBER: PrecClimber<Rule> = {
        use Assoc::*;
        use Rule::*;
        let op = |rule: Rule| Operator::new(rule, Left);
        PrecClimber::new(vec![
            op(operator_binary_push),
            op(operator_binary_or),
            op(operator_binary_and),
            op(operator_binary_eq) | op(operator_binary_neq),
            op(operator_binary_gt)
                | op(operator_binary_lt)
                | op(operator_binary_gte)
                | op(operator_binary_lte),
            op(operator_binary_add) | op(operator_binary_sub),
            op(operator_binary_mul) | op(operator_binary_div),
            op(operator_binary_ncl),
        ])
    };
}

fn build(pair: Pair<Rule>) -> Box<AstNode> {
    let span = pair.as_span();
    match pair.as_rule() {
        Rule::module => {
            let inner = pair.into_inner();
            AstNode::create(
                &span,
                AstNodeVariant::Module {
                    statements: inner.map(|child| build(child)).collect::<Vec<_>>(),
                },
            )
        }
        Rule::null => AstNode::create(&span, AstNodeVariant::Null),
        Rule::boolean => AstNode::create(
            &span,
            AstNodeVariant::Boolean {
                value: content(&pair) == "true",
            },
        ),
        Rule::number => AstNode::create(
            &span,
            AstNodeVariant::Number {
                value: content(&pair).parse::<f64>().unwrap(),
            },
        ),
        Rule::string => AstNode::create(
            &span,
            AstNodeVariant::String {
                value: SharedImmutable::new(
                    unescape(&next(&mut pair.into_inner()).as_str()).unwrap(),
                ),
            },
        ),
        Rule::identifier => AstNode::create(
            &span,
            AstNodeVariant::Identifier {
                name: SharedImmutable::new(content(&pair)),
            },
        ),
        Rule::list => AstNode::create(
            &span,
            AstNodeVariant::List {
                values: pair
                    .into_inner()
                    .map(|child| build(child))
                    .collect::<Vec<_>>(),
            },
        ),
        Rule::dict => AstNode::create(
            &span,
            AstNodeVariant::Dict {
                pairs: pair
                    .into_inner()
                    .map(|child| build(child))
                    .collect::<Vec<_>>(),
            },
        ),
        Rule::pair => {
            let mut inner = pair.into_inner();
            AstNode::create(
                &span,
                AstNodeVariant::Pair {
                    key: build(next(&mut inner)),
                    value: build(next(&mut inner)),
                },
            )
        }
        Rule::function => {
            let mut inner = pair.into_inner();
            AstNode::create(
                &span,
                AstNodeVariant::Function {
                    name: SharedImmutable::new(content(&next(&mut inner))),
                    parameters: next(&mut inner)
                        .into_inner()
                        .map(|parameter| SharedImmutable::new(content(&parameter)))
                        .collect::<Vec<_>>(),
                    block: build(next(&mut inner)),
                },
            )
        }
        Rule::lambda => {
            let mut inner = pair.into_inner();
            AstNode::create(
                &span,
                AstNodeVariant::Lambda {
                    parameters: next(&mut inner)
                        .into_inner()
                        .map(|parameter| SharedImmutable::new(content(&parameter)))
                        .collect::<Vec<_>>(),
                    body: build(next(&mut inner)),
                },
            )
        }
        Rule::function_statement => {
            let mut inner = pair.into_inner();
            AstNode::create(
                &span,
                AstNodeVariant::FunctionStatement {
                    function: build(next(&mut inner)),
                },
            )
        }
        Rule::variable_declaration_statement => {
            let mut inner = pair.into_inner();
            AstNode::create(
                &span,
                AstNodeVariant::VariableDeclarationStatement {
                    name: SharedImmutable::new(content(&next(&mut inner))),
                    value: build(next(&mut inner)),
                },
            )
        }
        Rule::variable_assignment_statement => {
            let mut inner = pair.into_inner();
            AstNode::create(
                &span,
                AstNodeVariant::VariableAssignmentStatement {
                    name: SharedImmutable::new(content(&next(&mut inner))),
                    operator: AssignmentOperator::from_rule(&next(&mut inner).as_rule()),
                    value: build(next(&mut inner)),
                },
            )
        }
        Rule::chain_assignment_statement => {
            let mut inner = pair.into_inner();
            let target = build(next(&mut inner));

            match target.variant() {
                AstNodeVariant::Index { .. } => AstNode::create(
                    &span,
                    AstNodeVariant::IndexAssignmentStatement {
                        index: target,
                        operator: AssignmentOperator::from_rule(&next(&mut inner).as_rule()),
                        value: build(next(&mut inner)),
                    },
                ),
                _ => AstNode::create(&span, AstNodeVariant::Unknown),
            }
        }
        Rule::loop_statement => {
            let mut inner = pair.into_inner();
            AstNode::create(
                &span,
                AstNodeVariant::LoopStatement {
                    block: build(next(&mut inner)),
                },
            )
        }
        Rule::while_statement => {
            let mut inner = pair.into_inner();
            AstNode::create(
                &span,
                AstNodeVariant::WhileStatement {
                    condition: build(next(&mut inner)),
                    block: build(next(&mut inner)),
                },
            )
        }
        Rule::return_statement => {
            let mut inner = pair.into_inner();
            AstNode::create(
                &span,
                AstNodeVariant::ReturnStatement {
                    value: inner.next().map(|value| build(value)),
                },
            )
        }
        Rule::break_statement => AstNode::create(&span, AstNodeVariant::BreakStatement),
        Rule::continue_statement => AstNode::create(&span, AstNodeVariant::ContinueStatement),
        Rule::echo_statement => {
            let mut inner = pair.into_inner();
            AstNode::create(
                &span,
                AstNodeVariant::EchoStatement {
                    value: build(next(&mut inner)),
                },
            )
        }
        Rule::expression_statement => {
            let mut inner = pair.into_inner();
            AstNode::create(
                &span,
                AstNodeVariant::ExpressionStatement {
                    expression: build(next(&mut inner)),
                },
            )
        }
        Rule::if_statement => {
            let mut inner = pair.into_inner();
            AstNode::create(
                &span,
                AstNodeVariant::IfStatement {
                    condition: build(next(&mut inner)),
                    block: build(next(&mut inner)),
                    else_statement: inner.next().map(build),
                },
            )
        }
        Rule::else_statement => {
            let mut inner = pair.into_inner();
            AstNode::create(
                &span,
                AstNodeVariant::ElseStatement {
                    next: build(next(&mut inner)),
                },
            )
        }
        Rule::block => AstNode::create(
            &span,
            AstNodeVariant::Block {
                statements: pair
                    .into_inner()
                    .map(|child| build(child))
                    .collect::<Vec<_>>(),
            },
        ),
        Rule::function_block => AstNode::create(
            &span,
            AstNodeVariant::FunctionBlock {
                statements: pair
                    .into_inner()
                    .map(|child| build(child))
                    .collect::<Vec<_>>(),
            },
        ),
        Rule::wrapped => {
            let mut inner = pair.into_inner();
            AstNode::create(
                &span,
                AstNodeVariant::Wrapped {
                    value: build(next(&mut inner)),
                },
            )
        }
        Rule::binary_operations => {
            let inner = pair.into_inner();
            CLIMBER.climb(
                inner,
                |pair: Pair<Rule>| build(pair),
                |left: Box<AstNode>, operator: Pair<Rule>, right: Box<AstNode>| {
                    let operator = BinaryOperator::from_rule(&operator.as_rule());
                    AstNode::create(
                        &span,
                        AstNodeVariant::BinaryOperation {
                            left,
                            operator,
                            right,
                        },
                    )
                },
            )
        }
        Rule::chain => {
            let mut inner = pair.into_inner();
            let target = build(next(&mut inner));

            inner.fold(target, |target, current| match current.as_rule() {
                Rule::index => AstNode::create(
                    &current.as_span(),
                    AstNodeVariant::Index {
                        target,
                        index: build(next(&mut current.into_inner())),
                    },
                ),
                Rule::call => AstNode::create(
                    &current.as_span(),
                    AstNodeVariant::Call {
                        target,
                        arguments: next(&mut current.into_inner())
                            .into_inner()
                            .map(|argument| build(argument))
                            .collect::<Vec<_>>(),
                    },
                ),
                _ => unreachable!(),
            })
        }
        _ => {
            panic!("Unrecognized rule for build: {:?}", pair.as_rule());
        }
    }
}

fn content<'a>(pair: &Pair<'a, Rule>) -> String {
    pair.as_str().trim().into()
}

fn next<'a>(pairs: &mut Pairs<'a, Rule>) -> Pair<'a, Rule> {
    pairs.next().unwrap()
}
