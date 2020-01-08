use pest::Parser;
use pest_derive::*;

#[derive(Parser)]
#[grammar = "assembler.pest"]
struct AssemblerParser;

use crate::instruction::Instruction;
use pest::iterators::Pairs;

pub fn create_program(text: &str) -> Result<Vec<Instruction>, pest::error::Error<Rule>> {
    let mut assembler: Pairs<Rule> = AssemblerParser::parse(Rule::assembler, text)?;

    let instructions = assembler.next().unwrap().into_inner()
        // .inspect(|pair| println!("{:?}", pair))
        .filter(|pair| { let r = pair.as_rule();  r != Rule::comment && r != Rule::EOI })
        .map(|pair| {
            let mut pairs = pair.into_inner();
            let instr = pairs.next().unwrap();
            let rule = instr.as_rule();
            let get_index = || instr.into_inner().next().unwrap().as_str().parse().unwrap();
            match rule {
                Rule::get => Instruction::Get,
                Rule::put => Instruction::Put,
                Rule::load => Instruction::Load(get_index()),
                Rule::loadi => Instruction::Loadi(get_index()),
                Rule::store => Instruction::Store(get_index()),
                Rule::storei => Instruction::Storei(get_index()),
                Rule::add => Instruction::Add(get_index()),
                Rule::sub => Instruction::Sub(get_index()),
                Rule::shift => Instruction::Shift(get_index()),
                Rule::inc => Instruction::Inc,
                Rule::dec => Instruction::Dec,
                Rule::jump => Instruction::Jump(get_index()),
                Rule::jpos => Instruction::Jpos(get_index()),
                Rule::jzero => Instruction::Jzero(get_index()),
                Rule::jneg => Instruction::Jneg(get_index()),
                Rule::halt => Instruction::Halt,
                _ => unreachable!(),
            }
        })
        .collect();

    Ok(instructions)
}
