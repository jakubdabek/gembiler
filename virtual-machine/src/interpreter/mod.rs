#![allow(dead_code)]

use crate::instruction::Instruction;
use std::collections::BTreeMap;

#[cfg(feature = "bignum")]
use num_bigint::{BigInt, Sign, RandBigInt};
#[cfg(feature = "bignum")]
use num_traits::cast::ToPrimitive;
use std::rc::Rc;
use std::cell::RefCell;
use crate::interpreter::world::World;
use std::fmt::{self, Debug, Formatter};
use std::convert::TryInto;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    UninitializedMemoryAccess,
    InstructionPointerOutOfBound,
    WorldError(world::Error),
}

impl From<world::Error> for Error {
    fn from(err: world::Error) -> Self {
        Error::WorldError(err)
    }
}

#[cfg(not(feature = "bignum"))]
pub type MemoryValue = i64;
#[cfg(feature = "bignum")]
pub type MemoryValue = BigInt;

pub fn memval(v: i64) -> MemoryValue {
    v.into()
}

type Memory = BTreeMap<i64, MemoryValue>;
type IResult = Result<(), Error>;

pub mod world;
#[cfg(test)]
mod tests;

fn shift(a: &MemoryValue, b: &MemoryValue) -> MemoryValue {
    #[cfg(feature = "bignum")] {
        match b.sign() {
            Sign::Plus => a << b.to_usize().expect("SHIFT operand out of range"),
            Sign::Minus => a >> (-b).to_usize().expect("SHIFT operand out of range"),
            Sign::NoSign => a.clone()
        }
    }

    #[cfg(not(feature = "bignum"))] {
        match b.signum() {
            1 => a << b,
            -1 => a >> -b,
            0 => *a,
            _ => unreachable!(),
        }
    }
}

pub struct Interpreter {
    world: Rc<RefCell<dyn World<MemoryValue>>>,
    memory: Memory,
    cost: u64,
    instr_ptr: usize,
    program: Vec<Instruction>,
}

impl Debug for Interpreter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Interpreter {{")?;
        writeln!(f, "    memory: {:?}", self.memory)?;
        writeln!(f, "    {}: {:?}", self.instr_ptr, self.program.get(self.instr_ptr))?;
        writeln!(f, "}}")
    }
}

fn random_memory_value() -> MemoryValue {
    #[cfg(feature = "bignum")] {
        let mut rng = rand::thread_rng();
        rng.gen_bigint(16)
    }
    #[cfg(not(feature = "bignum"))] {
        rand::random()
    }
}

impl Interpreter {
    pub fn new(world: Rc<RefCell<dyn World<MemoryValue>>>, program: Vec<Instruction>) -> Interpreter {
        Interpreter {
            world,
            memory: {
                let mut map = BTreeMap::new();
                map.insert(0, random_memory_value());
                map
            },
            cost: 0,
            instr_ptr: 0,
            program,
        }
    }

    fn log(&self, args: fmt::Arguments) {
        self.world.borrow_mut().log(args);
    }

    fn get_initialized(&self, index: i64) -> Result<&MemoryValue, Error> {
        self.memory.get(&index).ok_or(Error::UninitializedMemoryAccess)
    }

    fn assign_indirect(&mut self, index: i64, indirect_index: i64) -> IResult {
        let value_index = self.get_initialized(indirect_index)?;

        #[cfg(feature = "bignum")]
        let value_index = &value_index.to_i64().expect("indirect index out of range");

        let value_index = *value_index;
        self.assign(index, value_index)
    }

    fn assign(&mut self, index: i64, value_index: i64) -> IResult {
        let value = self.get_initialized(value_index)?.clone();
        self.log(format_args!("assign: [{}] <- {}", index, &value));
        self.memory.insert(index, value);
        Ok(())
    }

    fn mutate<F: Fn(&MemoryValue) -> MemoryValue>(&mut self, index: i64, f: F) -> IResult {
        let value = self.get_initialized(index)?;
        let new_value = f(value);
        self.log(format_args!("mutate: [{}] <- f({})", index, value));
        self.memory.insert(index, new_value);

        Ok(())
    }

    fn mutate_bin<F: Fn(&MemoryValue, &MemoryValue) -> MemoryValue>(&mut self, index: i64, value_index: i64, f: F) -> IResult {
        let acc_value = self.get_initialized(index)?;
        let value = self.get_initialized(value_index)?;
        let new_value = f(acc_value, value);
        self.log(format_args!("mutate_bin: [{}] <- f({}, {})", index, acc_value, value));
        self.memory.insert(index, new_value);

        Ok(())
    }

    pub fn interpret(&mut self) -> Result<u64, Error> {
        loop {
            match self.interpret_single() {
                Ok(true) => {},
                Ok(false) => return Ok(self.cost),
                Err(error) => return Err(error),
            }
        }
    }

    pub fn interpret_single(&mut self) -> Result<bool, Error> {
        if let Some(instr) = self.program.get(self.instr_ptr) {
            let cost = instr.cost();
            match *instr {
                Instruction::Get => {
                    self.cost += cost;
                    let value = self.world.borrow_mut().get()?;
                    self.log(format_args!("{:?}", value));
                    self.memory.insert(0, value);
                    self.instr_ptr += 1;
                },
                Instruction::Put => {
                    self.cost += cost;
                    self.world.borrow_mut().put(&self.memory[&0]);
                    self.instr_ptr += 1;
                },
                Instruction::Load(arg) => {
                    self.cost += cost;
                    self.assign(0, arg.try_into().unwrap())?;
                    self.instr_ptr += 1;
                },
                Instruction::Loadi(arg) => {
                    self.cost += cost;
                    self.assign_indirect(0, arg.try_into().unwrap())?;
                    self.instr_ptr += 1;
                },
                Instruction::Store(arg) => {
                    self.cost += cost;
                    self.assign(arg.try_into().unwrap(), 0)?;
                    self.instr_ptr += 1;
                },
                Instruction::Storei(arg) => {
                    self.cost += cost;
                    self.assign(0, arg.try_into().unwrap())?;
                    self.instr_ptr += 1;
                },
                Instruction::Add(arg) => {
                    self.cost += cost;
                    self.log(format_args!("ADD {}", arg));
                    self.mutate_bin(0, arg.try_into().unwrap(), |a, b| a + b)?;
                    self.instr_ptr += 1;
                },
                Instruction::Sub(arg) => {
                    self.cost += cost;
                    self.log(format_args!("SUB {}", arg));
                    self.mutate_bin(0, arg.try_into().unwrap(), |a, b| a - b)?;
                    self.instr_ptr += 1;
                },
                Instruction::Shift(arg) => {
                    self.cost += cost;
                    self.log(format_args!("SHIFT {}", arg));
                    self.mutate_bin(0, arg.try_into().unwrap(), shift)?;
                    self.instr_ptr += 1;
                },
                Instruction::Inc => {
                    self.cost += cost;
                    self.log(format_args!("INC"));
                    self.mutate(0, |a| a + 1)?;
                    self.instr_ptr += 1;
                },
                Instruction::Dec => {
                    self.cost += cost;
                    self.log(format_args!("DEC"));
                    self.mutate(0, |a| a - 1)?;
                    self.instr_ptr += 1;
                },
                Instruction::Jump(arg) => {
                    self.cost += cost;
                    self.instr_ptr = arg.try_into().unwrap();
                },
                Instruction::Jpos(arg) => {
                    self.cost += cost;
                    if self.memory[&0] > 0.into() {
                        self.instr_ptr = arg.try_into().unwrap();
                    } else {
                        self.instr_ptr += 1;
                    }
                },
                Instruction::Jzero(arg) => {
                    self.cost += cost;
                    if self.memory[&0] == 0.into() {
                        self.instr_ptr = arg.try_into().unwrap();
                    } else {
                        self.instr_ptr += 1;
                    }
                },
                Instruction::Jneg(arg) => {
                    self.cost += cost;
                    if self.memory[&0] < 0.into() {
                        self.instr_ptr = arg.try_into().unwrap();
                    } else {
                        self.instr_ptr += 1;
                    }
                },
                Instruction::Halt => { return Ok(false); },
            }

            Ok(true)
        } else {
            Err(Error::InstructionPointerOutOfBound)
        }
    }

    pub fn iter(self) -> InterpreterIter {
        InterpreterIter::new(self)
    }
}

pub struct InterpreterIter {
    interpreter: Interpreter,
}

impl InterpreterIter {
    fn new(interpreter: Interpreter) -> InterpreterIter {
        InterpreterIter {
            interpreter,
        }
    }
}

impl Iterator for InterpreterIter {
    type Item = IResult;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.interpreter.interpret_single();
//        println!("in next: {:?}{:?}", self.interpreter, res);
        match res {
            Ok(true) => Some(Ok(())),
            Ok(false) => None,
            Err(err) => Some(Err(err)),
        }
    }
}

impl IntoIterator for Interpreter {
    type Item = <InterpreterIter as Iterator>::Item;
    type IntoIter = InterpreterIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
