use self::base::AstModule;
use self::error::ParseError;
use self::grammar::{parse, GrammarRule, ParseContext};

pub mod base;
pub mod error;
pub mod expression;
pub mod location;
pub mod node;
pub mod operator;
pub mod statement;

mod grammar;
mod unescape;

#[derive(Debug)]
pub struct Ast<T> {
    root: T,
}

impl<T> Ast<T> {
    pub fn parse_module(code: &str) -> Result<Ast<AstModule>, ParseError> {
        let root = AstModule::parse(
            parse(GrammarRule::module, code)
                .map_err(|error| ParseError::from_grammar_error(error))?,
            &ParseContext::default(),
        );
        Ok(Ast { root })
    }

    pub fn root(&self) -> &T {
        &self.root
    }
}
