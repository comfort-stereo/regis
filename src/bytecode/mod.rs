mod builder;
mod bytecode;
mod instruction;
mod procedure;

use crate::ast::base::AstModule;
use crate::ast::Ast;

pub use builder::Builder;
pub use bytecode::Bytecode;
pub use instruction::Instruction;
pub use procedure::Procedure;

pub fn compile_module(module: &Ast<AstModule>) -> Bytecode {
    let mut builder = Builder::new();
    builder.emit_module(module.root());
    builder.build()
}
