use parser::ParseError;

use self::base::AstModule;
use self::parser::{parse, ParseContext, ParseRule};

pub mod base;
pub mod expression;
pub mod node;
pub mod operator;
pub mod parser;
pub mod statement;

#[derive(Debug)]
pub struct Ast<T> {
    root: T,
}

impl<T> Ast<T> {
    pub fn parse_module(code: &str) -> Result<Ast<AstModule>, ParseError> {
        let root = AstModule::parse(parse(ParseRule::module, code)?, &ParseContext::default());
        Ok(Ast { root })
    }

    pub fn root(&self) -> &T {
        &self.root
    }
}
