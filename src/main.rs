extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;
use std::fs;

#[derive(Parser)]
#[grammar = "program.pest"]
struct ProgramParser;

fn debug_file(filename: &str) {
    let program_text = fs::read_to_string(filename).expect("unable to read file");
    let program = ProgramParser::parse(Rule::program, &program_text);

    println!("Result for {}: {:?}", filename, program);
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
