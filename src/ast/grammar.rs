use pest::error::{Error, ErrorVariant};
use pest::iterators::{Pair, Pairs};
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::Parser;

use super::node::AstNodeInfo;

#[derive(Parser)]
#[grammar = "ast/grammar.pest"]
pub struct GrammarParser;

pub type GrammarRule = Rule;
pub type GrammarPair<'a> = Pair<'a, GrammarRule>;
pub type GrammarPairs<'a> = Pairs<'a, GrammarRule>;
pub type GrammarPrecClimber = PrecClimber<GrammarRule>;
pub type GrammarAssoc = Assoc;
pub type GrammarError = Error<GrammarRule>;
pub type GrammarErrorVariant = ErrorVariant<GrammarRule>;
pub type GrammarOperator = Operator<GrammarRule>;

#[derive(Debug)]
pub struct ParseContext;

impl Default for ParseContext {
    fn default() -> Self {
        Self {}
    }
}

pub fn parse(rule: GrammarRule, code: &str) -> Result<GrammarPair, GrammarError> {
    let pairs = GrammarParser::parse(rule, code)?.into_iter();
    for pair in pairs {
        return Ok(pair);
    }

    unreachable!();
}

pub fn content<'a>(pair: &GrammarPair<'a>) -> String {
    pair.as_str().trim().into()
}

pub fn extract<'a>(pair: GrammarPair<'a>) -> (AstNodeInfo, GrammarPairs<'a>) {
    let info = AstNodeInfo::new(&pair);
    let inner = pair.into_inner();
    (info, inner)
}
