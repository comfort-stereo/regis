pub mod base;
pub mod error;
pub mod expression;
pub mod location;
pub mod node;
pub mod operator;
pub mod statement;

mod grammar;
mod unescape;

use base::AstModule;
use error::ParseError;
use grammar::{parse, GrammarRule, ParseContext};

#[derive(Debug)]
pub struct Ast<T> {
    root: T,
}

impl<T> Ast<T> {
    pub fn parse_module(code: &str) -> Result<Ast<AstModule>, ParseError> {
        let root = AstModule::parse(
            parse(GrammarRule::module, code).map_err(ParseError::from_grammar_error)?,
            &ParseContext::default(),
        );
        Ok(Ast { root })
    }

    pub fn root(&self) -> &T {
        &self.root
    }
}
