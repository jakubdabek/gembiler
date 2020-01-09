use std::rc::Rc;
use std::cell::RefCell;
use crate::interpreter::{Interpreter, MemoryValue, Error};
use crate::interpreter::world::{self, MemoryWorld};
use crate::instruction::Instruction;
use crate::interpreter;

type TestWorld = MemoryWorld<MemoryValue>;
type TestWorldRc = Rc<RefCell<TestWorld>>;

fn get_world(inputs: Vec<MemoryValue>) -> TestWorldRc {
    Rc::new(RefCell::new(MemoryWorld::new(inputs)))
}

fn interpret(inputs: Vec<MemoryValue>, program: Vec<Instruction>) -> (TestWorldRc, Result<u64, Error>) {
    let world = get_world(inputs);
    let mut interpreter = Interpreter::new(world::upcast(Rc::clone(&world)), program);

    (world, interpreter.interpret())
}

#[test]
fn empty_program() {
    let (_, result) = interpret(vec![], vec![]);
    assert_eq!(result, Err(Error::InstructionPointerOutOfBound));
}

#[test]
fn halt() {
    let (_, result) = interpret(vec![], vec![Instruction::Halt]);
    assert_eq!(result, Ok(0));
}

#[test]
fn uninitialized() {
    let (_, result) = interpret(vec![], vec![
        Instruction::Load(1),
        Instruction::Halt,
    ]);

    assert_eq!(result, Err(Error::UninitializedMemoryAccess));
}

#[test]
fn simple() {
    let program = vec![
        Instruction::Sub(0),
        Instruction::Halt,
    ];

    let cost = program.iter().map(|i| i.cost()).sum();

    let (_, result) = interpret(vec![], program);

    assert_eq!(result, Ok(cost))
}

#[test]
fn simple_output() {
    let program = vec![
        Instruction::Sub(0),
        Instruction::Put,
        Instruction::Halt,
    ];

    let cost = program.iter().map(|i| i.cost()).sum();

    let (world, result) = interpret(vec![], program);

    assert_eq!(result, Ok(cost));
    assert_eq!(world.borrow().output(), &[0.into()]);
}

#[test]
fn simple_io() {
    let program = vec![
        Instruction::Get,
        Instruction::Put,
        Instruction::Halt,
    ];

    let cost = program.iter().map(|i| i.cost()).sum();
    let val = interpreter::memval(42);
    let inputs = vec![val];
    let outputs = inputs.clone();

    let (world, result) = interpret(inputs, program);

    assert_eq!(result, Ok(cost));
    assert_eq!(world.borrow().output(), &*outputs);
}

#[test]
fn simple_io2() {
    let program = vec![
        Instruction::Get,
        Instruction::Put,
        Instruction::Put,
        Instruction::Halt,
    ];

    let cost = program.iter().map(|i| i.cost()).sum();
    let val = interpreter::memval(42);
    let inputs = vec![val.clone()];
    let outputs = vec![val.clone(), val.clone()];

    let (world, result) = interpret(inputs, program);

    assert_eq!(result, Ok(cost));
    assert_eq!(world.borrow().output(), &*outputs);
}

#[test]
fn simple_arithmetic() {
    let program = vec![
        Instruction::Get,
        Instruction::Inc,
        Instruction::Put,
        Instruction::Halt,
    ];

    let cost = program.iter().map(|i| i.cost()).sum();
    let val = interpreter::memval(42);
    let inputs = vec![val.clone()];
    let outputs = vec![val + 1];

    let (world, result) = interpret(inputs, program);

    assert_eq!(result, Ok(cost));
    assert_eq!(world.borrow().output(), &*outputs);
}

#[test]
fn simple_arithmetic2() {
    let program = vec![
        Instruction::Get,
        Instruction::Inc,
        Instruction::Inc,
        Instruction::Add(0),
        Instruction::Dec,
        Instruction::Put,
        Instruction::Halt,
    ];

    let cost = program.iter().map(|i| i.cost()).sum();
    let val = interpreter::memval(42);
    let inputs = vec![val.clone()];
    let outputs = vec![2 * val + 3];

    let (world, result) = interpret(inputs, program);

    assert_eq!(result, Ok(cost));
    assert_eq!(world.borrow().output(), &*outputs);
}
