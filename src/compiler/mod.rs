use builder::Builder;
use bytecode::Bytecode;

use crate::ast::base::AstModule;

pub mod base;
pub mod builder;
pub mod bytecode;
pub mod emit;
pub mod expression;
pub mod operator;
pub mod statement;

pub fn compile_module(module: &AstModule) -> Bytecode {
    let mut builder = Builder::new();
    builder.emit_module(&module);
    builder.build()
}
