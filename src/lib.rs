#[macro_use]
extern crate lazy_static;
extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate indexmap;
extern crate uuid;

pub mod ast;
pub mod bytecode;
pub mod interpreter;
pub mod shared;
pub mod vm;
