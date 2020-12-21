#[macro_use]
extern crate lazy_static;
extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate indexmap;
extern crate uuid;

mod ast;
mod compiler;
mod dict;
mod function;
mod interpreter;
mod interpreter_error;
mod list;
mod oid;
mod shared;
mod unescape;
mod value;
mod value_type;

use crate::interpreter::Interpreter;
use std::{env, fs, process};

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let path = args.get(1).unwrap_or_else(|| {
        println!("ERROR: Provide a file to execute.");
        process::exit(1);
    });
    let path = fs::canonicalize(path).unwrap_or_else(|_| {
        println!("ERROR: Path does not exist.");
        process::exit(2);
    });
    let code = fs::read_to_string(path).unwrap_or_else(|_| {
        println!("ERROR: Cannot read file as text.");
        process::exit(3);
    });

    let mut vm = Interpreter::new();
    match vm.run_module(&code) {
        Ok(()) => {}
        Err(error) => {
            println!("{}", error);
        }
    }
}
