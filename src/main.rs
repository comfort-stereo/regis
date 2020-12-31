use std::{env, process};

use regis::interpreter::Interpreter;
use regis::path::CanonicalPath;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let path = args.get(1).unwrap_or_else(|| {
        println!("ERROR: Provide a file to execute.");
        process::exit(1);
    });
    let path = CanonicalPath::from(path).unwrap_or_else(|| {
        println!("ERROR: Specified file path does not exist.");
        process::exit(1);
    });

    let mut interpreter = Interpreter::new(path.clone());
    if let Err(error) = interpreter.load_module(&path) {
        println!("{}", error.show());
        process::exit(1);
    }
}
