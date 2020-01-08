extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::fs;
use std::rc::Rc;
use std::cell::RefCell;

mod instruction;
mod interpreter;
mod parser;

use interpreter::{Interpreter, World, ConsoleWorld};

fn main() {
    let text = fs::read_to_string("test-data/program0.mr").expect("couldn't read file");

    // let world = Rc::new(RefCell::new(MemoryWorld::new(vec![5])));
    let world = Rc::new(RefCell::new(ConsoleWorld::new(false)));
    let result: Result<(), String> = parser::create_program(&text)
        .map_err(|e| format!("parse error: {}", e))
        .map(|instructions| Interpreter::new(Rc::clone(&world) as Rc<RefCell<dyn World>>, instructions))
        .and_then(|interpreter| {
            interpreter.iter().collect::<Result<_, _>>()
                .map_err(|s| s.to_owned())
        });

    println!("{:?}", result);
    // println!("{}", world.borrow().output());
}
