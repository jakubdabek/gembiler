extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::fs;

mod parser;
mod instruction;

use instruction::Instruction;
use std::collections::BTreeMap;
use std::io::{stdin, BufRead};
use std::convert::TryInto;
use rand::prelude::*;
use std::fmt::{Debug, Formatter, Error};
use std::str::FromStr;

#[cfg(feature = "bignum")]
use num_bigint::BigInt;

const UNINITIALIZED_ERROR: &'static str = "access to uninitialized memory";
const INVALID_NUMBER_FORMAT_ERROR: &'static str = "invalid number format";

#[cfg(not(feature = "bignum"))]
type Memory = BTreeMap<i64, i64>;
#[cfg(feature = "bignum")]
type Memory = BTreeMap<i64, BigInt>;
type IResult = Result<(), &'static str>;

struct Interpreter {
    memory: Memory,
    cost: u64,
    instr_ptr: usize,
    program: Vec<Instruction>,
}

impl Debug for Interpreter {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        writeln!(f, "Interpreter {{")?;
        writeln!(f, "    memory: {:?}", self.memory)?;
        writeln!(f, "    {}: {:?}", self.instr_ptr, self.program.get(self.instr_ptr))?;
        writeln!(f, "}}")
    }
}

fn parse_line<F: FromStr>() -> Result<F, F::Err> {
    let mut buf = String::new();
    stdin().lock().read_line(&mut buf).expect("error reading stdin");

    buf.trim_end_matches('\n').parse()
}

impl Interpreter {
    fn new(program: Vec<Instruction>) -> Interpreter {
        Interpreter {
            memory: {
                let mut map = BTreeMap::new();
                map.insert(0, random());
                map
            },
            cost: 0,
            instr_ptr: 0,
            program,
        }
    }

    fn assign(&mut self, index: i64, value_index: i64) -> IResult {
        let value = *self.memory.get(&value_index).ok_or(UNINITIALIZED_ERROR)?;
        self.memory.insert(index, value);
        Ok(())
    }

    fn mutate<F: Fn(i64) -> i64>(&mut self, index: i64, f: F) -> IResult {
        if let Some(mem) = self.memory.get_mut(&index) {
            *mem = f(*mem);
            Ok(())
        } else {
            Err("access to uninitialized memory")
        }
    }

    fn mutate_bin<F: Fn(i64, i64) -> i64>(&mut self, index: i64, value_index: i64, f: F) -> IResult {
        let value = *self.memory.get(&value_index).ok_or(UNINITIALIZED_ERROR)?;
        if let Some(mem) = self.memory.get_mut(&index) {
            *mem = f(*mem, value);
            Ok(())
        } else {
            Err("access to uninitialized memory")
        }
    }

    fn interpret_single(&mut self) -> Result<bool, &'static str> {
        if let Some(instr) = self.program.get(self.instr_ptr) {
            match *instr {
                Instruction::Get => {
                    self.cost += 100;
                    let parsed_value = parse_line();
                    println!("{:?}", parsed_value);
                    self.memory.insert(0, parsed_value.map_err(|_| INVALID_NUMBER_FORMAT_ERROR)?);
                    self.instr_ptr += 1;
                },
                Instruction::Put => {
                    self.cost += 100;
                    println!("{}", self.memory[&0]);
                    self.instr_ptr += 1;
                },
                Instruction::Load(arg) => {
                    self.cost += 10;
                    self.assign(0, arg.try_into().unwrap())?;
                    self.instr_ptr += 1;
                },
                Instruction::Loadi(arg) => {
                    self.cost += 20;
                    let ind = *self.memory.get(&arg.try_into().unwrap()).ok_or(UNINITIALIZED_ERROR)?;
                    self.assign(0, ind)?;
                    self.instr_ptr += 1;
                },
                Instruction::Store(arg) => {
                    self.cost += 10;
                    self.assign(arg.try_into().unwrap(), 0)?;
                    self.instr_ptr += 1;
                },
                Instruction::Storei(arg) => {
                    self.cost += 20;
                    let ind = *self.memory.get(&arg.try_into().unwrap()).ok_or(UNINITIALIZED_ERROR)?;
                    self.assign(0, ind)?;
                    self.instr_ptr += 1;
                },
                Instruction::Add(arg) => {
                    self.cost += 10;
                    self.mutate_bin(0, arg.try_into().unwrap(), |a, b| a + b)?;
                    self.instr_ptr += 1;
                },
                Instruction::Sub(arg) => {
                    self.cost += 10;
                    self.mutate_bin(0, arg.try_into().unwrap(), |a, b| a - b)?;
                    self.instr_ptr += 1;
                },
                Instruction::Shift(arg) => {
                    self.cost += 5;
                    self.mutate_bin(0, arg.try_into().unwrap(), |a, b| a << b)?;
                    self.instr_ptr += 1;
                },
                Instruction::Inc => {
                    self.cost += 1;
                    self.mutate(0, |a| a + 1)?;
                    self.instr_ptr += 1;
                },
                Instruction::Dec => {
                    self.cost += 1;
                    self.mutate(0, |a| a - 1)?;
                    self.instr_ptr += 1;
                },
                Instruction::Jump(arg) => {
                    self.cost += 1;
                    self.instr_ptr = arg.try_into().unwrap();
                },
                Instruction::Jpos(arg) => {
                    self.cost += 1;
                    if self.memory[&0] > 0 {
                        self.instr_ptr = arg.try_into().unwrap();
                    } else {
                        self.instr_ptr += 1;
                    }
                },
                Instruction::Jzero(arg) => {
                    self.cost += 1;
                    if self.memory[&0] == 0 {
                        self.instr_ptr = arg.try_into().unwrap();
                    } else {
                        self.instr_ptr += 1;
                    }
                },
                Instruction::Jneg(arg) => {
                    self.cost += 1;
                    if self.memory[&0] < 0 {
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

    fn iter(self) -> InterpreterIter {
        InterpreterIter::new(self)
    }
}

struct InterpreterIter {
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

fn main() {
    let text = fs::read_to_string("test-data/program0.mr").expect("couldn't read file");
    let result: Result<(), String> = parser::create_program(&text)
        .map_err(|e| format!("parse error: {}", e))
        .map(|instructions| Interpreter::new(instructions))
        .and_then(|interpreter| {
            interpreter.iter().collect::<Result<_, _>>()
                .map_err(|s| s.to_owned())
        });

    println!("{:?}", result);
}
