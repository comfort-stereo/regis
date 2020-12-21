use super::node::AstNodeInfo;
use super::parser::{inner, ParseContext, ParsePair, ParseRule};
use super::statement::AstStatementVariant;

#[derive(Debug)]
pub struct AstModule {
    pub info: AstNodeInfo,
    pub statements: Vec<AstStatementVariant>,
}

impl AstModule {
    pub fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::module);
        let (info, inner) = inner(pair);
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
    pub fn parse(pair: ParsePair, context: &ParseContext) -> Self {
        assert_eq!(pair.as_rule(), ParseRule::block);
        let (info, inner) = inner(pair);
        Self {
            info,
            statements: inner
                .map(|statement| AstStatementVariant::parse(statement, context))
                .collect(),
        }
    }
}
