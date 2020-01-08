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
use std::fmt::{Debug, Formatter};
use std::fmt;
use std::convert::TryInto;

const UNINITIALIZED_MEMORY_ACCESS: &str = "access to uninitialized memory";

#[cfg(not(feature = "bignum"))]
pub type MemoryValue = i64;
#[cfg(feature = "bignum")]
pub type MemoryValue = BigInt;

type Memory = BTreeMap<i64, MemoryValue>;
type IResult = Result<(), &'static str>;

pub mod world;

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

    fn log<F: Fn() -> String>(&self, f: F) {
        self.world.borrow_mut().log(&f());
    }

    fn get_initialized(&self, index: i64) -> Result<&MemoryValue, &'static str> {
        self.memory.get(&index).ok_or(UNINITIALIZED_MEMORY_ACCESS)
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
        self.log(|| format!("assign: [{}] <- {}", index, &value));
        self.memory.insert(index, value);
        Ok(())
    }

    fn mutate<F: Fn(&MemoryValue) -> MemoryValue>(&mut self, index: i64, f: F) -> IResult {
        let value = self.get_initialized(index)?;
        let new_value = f(value);
        self.log(|| format!("mutate: [{}] <- f({})", index, value));
        self.memory.insert(index, new_value);

        Ok(())
    }

    fn mutate_bin<F: Fn(&MemoryValue, &MemoryValue) -> MemoryValue>(&mut self, index: i64, value_index: i64, f: F) -> IResult {
        let acc_value = self.get_initialized(index)?;
        let value = self.get_initialized(value_index)?;
        let new_value = f(acc_value, value);
        self.log(|| format!("mutate_bin: [{}] <- f({}, {})", index, acc_value, value));
        self.memory.insert(index, new_value);

        Ok(())
    }

    pub fn interpret_single(&mut self) -> Result<bool, &'static str> {
        if let Some(instr) = self.program.get(self.instr_ptr) {
            match *instr {
                Instruction::Get => {
                    self.cost += 100;
                    let value = self.world.borrow_mut().get()?;
                    self.log(|| format!("{:?}", value));
                    self.memory.insert(0, value);
                    self.instr_ptr += 1;
                },
                Instruction::Put => {
                    self.cost += 100;
                    self.world.borrow_mut().put(&self.memory[&0]);
                    self.instr_ptr += 1;
                },
                Instruction::Load(arg) => {
                    self.cost += 10;
                    self.assign(0, arg.try_into().unwrap())?;
                    self.instr_ptr += 1;
                },
                Instruction::Loadi(arg) => {
                    self.cost += 20;
                    self.assign_indirect(0, arg.try_into().unwrap())?;
                    self.instr_ptr += 1;
                },
                Instruction::Store(arg) => {
                    self.cost += 10;
                    self.assign(arg.try_into().unwrap(), 0)?;
                    self.instr_ptr += 1;
                },
                Instruction::Storei(arg) => {
                    self.cost += 20;
                    self.assign(0, arg.try_into().unwrap())?;
                    self.instr_ptr += 1;
                },
                Instruction::Add(arg) => {
                    self.cost += 10;
                    self.log(|| format!("ADD {}", arg));
                    self.mutate_bin(0, arg.try_into().unwrap(), |a, b| a + b)?;
                    self.instr_ptr += 1;
                },
                Instruction::Sub(arg) => {
                    self.cost += 10;
                    self.log(|| format!("SUB {}", arg));
                    self.mutate_bin(0, arg.try_into().unwrap(), |a, b| a - b)?;
                    self.instr_ptr += 1;
                },
                Instruction::Shift(arg) => {
                    self.cost += 5;
                    self.log(|| format!("SHIFT {}", arg));
                    self.mutate_bin(0, arg.try_into().unwrap(), shift)?;
                    self.instr_ptr += 1;
                },
                Instruction::Inc => {
                    self.cost += 1;
                    self.log(|| "INC".to_owned());
                    self.mutate(0, |a| a + 1)?;
                    self.instr_ptr += 1;
                },
                Instruction::Dec => {
                    self.cost += 1;
                    self.log(|| "DEC".to_owned());
                    self.mutate(0, |a| a - 1)?;
                    self.instr_ptr += 1;
                },
                Instruction::Jump(arg) => {
                    self.cost += 1;
                    self.instr_ptr = arg.try_into().unwrap();
                },
                Instruction::Jpos(arg) => {
                    self.cost += 1;
                    if self.memory[&0] > 0.into() {
                        self.instr_ptr = arg.try_into().unwrap();
                    } else {
                        self.instr_ptr += 1;
                    }
                },
                Instruction::Jzero(arg) => {
                    self.cost += 1;
                    if self.memory[&0] == 0.into() {
                        self.instr_ptr = arg.try_into().unwrap();
                    } else {
                        self.instr_ptr += 1;
                    }
                },
                Instruction::Jneg(arg) => {
                    self.cost += 1;
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
            Err("instruction pointer out of bounds")
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
