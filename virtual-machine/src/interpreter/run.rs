use crate::instruction::Instruction;
use crate::interpreter::{MemoryValue, world, Interpreter, Error};
use std::rc::Rc;
use std::cell::RefCell;

pub fn run(instructions: Vec<Instruction>, input: Vec<MemoryValue>) -> Result<(u64, Vec<MemoryValue>), Error> {
    run_internal(instructions, input, false)
}

pub fn run_extended(instructions: Vec<Instruction>, input: Vec<MemoryValue>) -> Result<(u64, Vec<MemoryValue>), Error> {
    run_internal(instructions, input, true)
}

pub fn run_interactive(instructions: Vec<Instruction>, verbose: bool) -> Result<u64, Error> {
    let world = Rc::new(RefCell::new(world::ConsoleWorld::new(verbose)));
    let mut interpreter = Interpreter::new_debug(world::upcast(world), instructions, true);
    interpreter.interpret()
}

pub fn run_debug(instructions: Vec<Instruction>, input: Vec<MemoryValue>, extended: bool) -> (Result<(u64, Vec<MemoryValue>), Error>, Vec<String>) {
    let world = Rc::new(RefCell::new(world::MemoryWorld::new(input)));
    let mut interpreter = Interpreter::new_debug(world::upcast(Rc::clone(&world)), instructions.to_vec(), extended);
    let result = interpreter.interpret();
    let logs = world.borrow().logs().map(str::to_owned).collect();

    let result = result.map(|cost| {
        let output = world.borrow().output().to_vec();
        (cost, output)
    });

    (result, logs)
}

fn run_internal(instructions: Vec<Instruction>, input: Vec<MemoryValue>, extended: bool) -> Result<(u64, Vec<MemoryValue>), Error> {
    let world = Rc::new(RefCell::new(world::MemoryWorld::new(input)));
    let mut interpreter = if extended {
        Interpreter::new_extended(world::upcast(Rc::clone(&world)), instructions.to_vec())
    } else {
        Interpreter::new(world::upcast(Rc::clone(&world)), instructions.to_vec())
    };
    interpreter.interpret().map(|cost| (cost, world.borrow().output().to_vec()))
}
