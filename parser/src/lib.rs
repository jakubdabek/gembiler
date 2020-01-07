extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;
use std::fs;

#[derive(Parser)]
#[grammar = "program.pest"]
struct ProgramParser;

pub fn debug_file(filename: &str) {
    let program_text = fs::read_to_string(filename).expect("unable to read file");
    let program = ProgramParser::parse(Rule::program, &program_text);

    println!("Result for {}: {:?}", filename, program);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
