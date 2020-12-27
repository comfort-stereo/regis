use std::{env, fs, process};

use interpreter::Interpreter;
use regis::*;

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
    let source = fs::read_to_string(path).unwrap_or_else(|_| {
        println!("ERROR: Cannot read file as text.");
        process::exit(3);
    });

    let mut interpreter = Interpreter::new();
    match interpreter.run_module(&source) {
        Ok(()) => {}
        Err(error) => {
            println!("{}", error.show(&source));
            process::exit(4);
        }
    }
}
