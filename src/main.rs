#[macro_use]
extern crate lazy_static;
extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate uuid;

mod bytecode;
mod list;
mod parser;
mod shared;
mod value;
mod value_type;
mod vm;
mod vm_error;

use crate::bytecode::*;
use crate::parser::{parse, AstRoot};
use crate::vm::Vm;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let path = args.get(1).unwrap_or_else(|| {
        println!("ERROR: Provide a file to execute.");
        process::exit(1);
    });
    println!("{:?}", path);
    let path = fs::canonicalize(path).unwrap_or_else(|_| {
        println!("ERROR: Path does not exist.");
        process::exit(2);
    });
    let code = fs::read_to_string(path).unwrap_or_else(|_| {
        println!("ERROR: Cannot read file as text.");
        process::exit(3);
    });

    let ast = parse(AstRoot::Module, &code);

    // println!("{:#?}", ast);
    let chunk = compile(&ast.unwrap());
    println!("Chunk: {:#?}", chunk);
    let mut vm = Vm::new();
    match vm.run_chunk(chunk) {
        Ok(()) => {}
        Err(error) => {
            println!("ERROR: {}", error);
        }
    }
}
