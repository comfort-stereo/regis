use builder::Builder;
use bytecode::Bytecode;

use crate::ast::base::AstModule;
use crate::ast::Ast;

pub mod base;
pub mod builder;
pub mod bytecode;
pub mod expression;
pub mod operator;
pub mod statement;

pub fn compile_module(module: &Ast<AstModule>) -> Bytecode {
    let mut builder = Builder::new();
    builder.emit_module(module.root());
    builder.build()
}
