use super::grammar::{extract, GrammarPair, GrammarRule, ParseContext};
use super::node::AstNodeInfo;
use super::statement::AstStatementVariant;

#[derive(Debug)]
pub struct AstModule {
    pub info: AstNodeInfo,
    pub statements: Vec<AstStatementVariant>,
}

impl AstModule {
    pub fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::module);
        let (info, inner) = extract(pair);
        Self {
            info,
            statements: inner
                .map(|statement| AstStatementVariant::parse(statement, context))
                .collect(),
        }
    }
}

#[derive(Debug)]
pub struct AstBlock {
    pub info: AstNodeInfo,
    pub statements: Vec<AstStatementVariant>,
}

impl AstBlock {
    pub fn parse(pair: GrammarPair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), GrammarRule::block);
        let (info, inner) = extract(pair);
        Self {
            info,
            statements: inner
                .map(|statement| AstStatementVariant::parse(statement, context))
                .collect(),
        }
    }
}
