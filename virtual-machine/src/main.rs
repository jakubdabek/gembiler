use std::fs;
use std::rc::Rc;
use std::cell::RefCell;

mod instruction;
mod interpreter;
mod parser;

use crate::interpreter::{Interpreter, world};

fn main() {
    let text = fs::read_to_string("test-data/program0.mr").expect("couldn't read file");

    // let world = Rc::new(RefCell::new(world::MemoryWorld::new(vec![5])));
    let world = Rc::new(RefCell::new(world::ConsoleWorld::new(false)));
    let result: Result<(), String> = parser::create_program(&text)
        .map_err(|e| format!("parse error: {}", e))
        .map(|instructions| Interpreter::new(world::upcast(Rc::clone(&world)), instructions))
        .and_then(|interpreter| {
            interpreter.iter().collect::<Result<_, _>>()
                .map_err(|s| s.to_owned())
        });

    println!("{:?}", result);
    // println!("{}", world.borrow().output());
}
