use std::fs;
use std::rc::Rc;
use std::cell::RefCell;

mod instruction;
mod interpreter;
mod parser;

use crate::interpreter::{Interpreter, world};

#[derive(Debug, PartialEq, Eq)]
enum Error {
    ParseError(String),
    InterpretError(interpreter::Error),
}

fn main() {
    let text = fs::read_to_string("test-data/program0.mr").expect("couldn't read file");

    // let world = Rc::new(RefCell::new(world::MemoryWorld::new(vec![5])));
    let world = Rc::new(RefCell::new(world::ConsoleWorld::new(false)));
    let result: Result<u64, Error> = parser::create_program(&text)
        .map_err(|e| Error::ParseError(e.to_string()))
        .map(|instructions| {
            Interpreter::new(world::upcast(Rc::clone(&world)), instructions)
        })
        .and_then(|mut interpreter| {
            interpreter.interpret().map_err(|e| Error::InterpretError(e))
        });

    println!("{:?}", result);
    // println!("{}", world.borrow().output());
}
