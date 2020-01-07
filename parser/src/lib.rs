extern crate pest;
#[macro_use]
extern crate pest_derive;

mod ast;

use pest::Parser;
use std::{fs, error::Error};

#[derive(Parser)]
#[grammar = "program.pest"]
struct ProgramParser;

pub fn debug_file(filename: &str) {
    let program_text = fs::read_to_string(filename).expect("unable to read file");
    let program = ProgramParser::parse(Rule::program, &program_text);

    println!("Result for {}: {:?}", filename, program);
}

type AstResult = Result<ast::Program, Box<dyn Error>>;

pub fn parse_file<P: AsRef<std::path::Path>>(path: P) -> AstResult {
    let program_text = fs::read_to_string(path)?;
    parse_ast(&program_text)
}

pub fn parse_ast(text: &str) -> AstResult {
    Ok(ast::Program { declarations: None, commands: vec![] })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(parse_ast("").unwrap(), ast::Program { declarations: None, commands: vec![] });
    }
}
