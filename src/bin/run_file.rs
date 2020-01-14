use gembiler::code_generator::{intermediate, translator};
use virtual_machine::interpreter;
use std::env;

fn run_file(path: &str, debug: bool) {
    let program = parser::parse_file(path);

    match program {
        Ok(program) => {
            let context = intermediate::generate(&program).unwrap();
            let generator = translator::Generator::new(context);
            let result = interpreter::run_interactive(generator.translate(), debug);
            match result {
                Ok(cost) => {
                    println!("Run successful, cost: {}", cost);
                },
                Err(error) => {
                    println!("Interpreter error: {:?}", error);
                }
            }
        },
        Err(err) => println!("{}", err),
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let len = args.len();

    match len {
        len if len < 2 => println!("Usage: {} [-v] <filename>", args[0]),
        2 => run_file(args[1].as_str(), false),
        _ => run_file(args[2].as_str(), args[1].as_str() == "-v"),
    }
}
