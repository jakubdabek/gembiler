use crate::instruction::Instruction;
use crate::interpreter::{MemoryValue, world, Interpreter, Error};
use std::rc::Rc;
use std::cell::RefCell;

pub fn run(instructions: Vec<Instruction>, input: Vec<MemoryValue>) -> Result<(Vec<MemoryValue>, u64), Error> {
    let world = Rc::new(RefCell::new(world::MemoryWorld::new(input)));
    let mut interpreter = Interpreter::new(world::upcast(Rc::clone(&world)), instructions.to_vec());
    interpreter.interpret().map(|cost| (world.borrow().output().to_vec(), cost))
}
