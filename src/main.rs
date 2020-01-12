use parser::parse_file;

fn debug_file(path: &str) {
    let program = parse_file(path);

    match program {
        Ok(program) => println!("{:?}", program),
        Err(err) => println!("{}", err),
    }
}

fn main() {
    debug_file("test-data/program0.imp");
    debug_file("test-data/program1.imp");
    debug_file("test-data/program2.imp");

    debug_file("test-data/p1.imp");

    for i in 0..=8 {
        debug_file(&format!("test-data/error{}.imp", i));
    }
}
