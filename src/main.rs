use gembiler::code_generator::{intermediate, translator};
use virtual_machine::interpreter;

fn debug_file(path: &str) {
    let program = parser::parse_file(path);

    match program {
        Ok(program) => {
            let context = intermediate::generate(&program).unwrap();
            let generator = translator::Generator::new(context);
            let result = interpreter::run_interactive(generator.translate(), false);
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
    debug_file("test-data/program0.imp");
//    debug_file("test-data/program1.imp");
    debug_file("test-data/program2.imp");

//    debug_file("test-data/p1.imp");

    for i in 0..=8 {
        debug_file(&format!("test-data/error{}.imp", i));
    }
}
