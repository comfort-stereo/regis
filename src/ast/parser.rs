use pest::error::Error;
use pest::iterators::{Pair, Pairs};
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::Parser;

use super::base::AstModule;
use super::node::AstNodeInfo;

#[derive(Parser)]
#[grammar = "ast/grammar.pest"]
pub struct InnerParser;

pub type ParseRule = Rule;
pub type ParsePair<'a> = Pair<'a, ParseRule>;
pub type ParsePairs<'a> = Pairs<'a, ParseRule>;
pub type ParsePrecClimber = PrecClimber<ParseRule>;
pub type ParseAssoc = Assoc;
pub type ParseError = Error<ParseRule>;
pub type ParseOperator = Operator<ParseRule>;

#[derive(Debug)]
pub struct ParseContext {
    is_in_function: bool,
}

impl Default for ParseContext {
    fn default() -> Self {
        Self {
            is_in_function: false,
        }
    }
}

pub trait AstNode: Sized + Default {}

pub fn parse_module(code: &str) -> Result<AstModule, ParseError> {
    read(ParseRule::module, code).map(|root| AstModule::parse(root, &ParseContext::default()))
}

fn read(rule: Rule, code: &str) -> Result<ParsePair, ParseError> {
    let pairs = InnerParser::parse(rule, code)?.into_iter();
    for pair in pairs {
        return Ok(pair);
    }

    unreachable!();
}

pub fn content<'a>(pair: &ParsePair<'a>) -> String {
    pair.as_str().trim().into()
}

pub fn inner<'a>(pair: ParsePair<'a>) -> (AstNodeInfo, ParsePairs<'a>) {
    let info = AstNodeInfo::new(&pair);
    let inner = pair.into_inner();
    (info, inner)
}
