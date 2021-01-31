use std::{env, process};

use regis::interpreter::Interpreter;
use regis::source::CanonicalPath;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let first = args.get(1).unwrap_or_else(|| {
        println!("ERROR: Provide a file to execute.");
        process::exit(1);
    });
    let path = CanonicalPath::from(first).unwrap_or_else(|| {
        println!("ERROR: Specified file path does not exist.");
        process::exit(1);
    });

    let mut interpreter = Interpreter::new(path.clone());
    if let Err(error) = interpreter.load_module(&path) {
        if let Ok(source) = path.read() {
            println!("{}", error.show(Some(&source)));
        } else {
            println!("{}", error.show(None));
        }

        process::exit(1);
    }
}
