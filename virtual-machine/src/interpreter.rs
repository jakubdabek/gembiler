use crate::instruction::Instruction;
use std::collections::BTreeMap;
use std::io::{self, BufRead, Write};
use std::convert::TryInto;
use std::fmt::{Debug, Formatter, Write as _, Error};
use std::str::FromStr;
use std::rc::Rc;
use std::cell::RefCell;

#[cfg(feature = "bignum")]
use num_bigint::{BigInt, Sign, RandBigInt};
#[cfg(feature = "bignum")]
use num_traits::cast::ToPrimitive;

const UNINITIALIZED_ERROR: &str = "access to uninitialized memory";
const INVALID_NUMBER_FORMAT_ERROR: &str = "invalid number format";

#[cfg(not(feature = "bignum"))]
type MemoryValue = i64;
#[cfg(feature = "bignum")]
type MemoryValue = BigInt;

type Memory = BTreeMap<i64, MemoryValue>;
type IResult = Result<(), &'static str>;

fn shift(a: &MemoryValue, b: &MemoryValue) -> MemoryValue {
    #[cfg(feature = "bignum")] {
        match b.sign() {
            Sign::Plus => a << b.to_usize().expect("SHIFT operand out of range"),
            Sign::Minus => a >> (-b).to_usize().expect("SHIFT operand out of range"),
            Sign::NoSign => a.clone()
        }
    }

    #[cfg(not(feature = "bignum"))] {
        if b > &0 {
            a << b
        } else if b < &0 {
            a >> -b
        } else {
            *a
        }
    }
}

pub trait World {
    fn get(&mut self) -> Result<MemoryValue, &'static str>;
    fn put(&mut self, val: &MemoryValue);
    fn log(&mut self, message: &str);
}

pub struct ConsoleWorld {
    verbose: bool,
}

impl ConsoleWorld {
    pub fn new(verbose: bool) -> ConsoleWorld {
        ConsoleWorld {
            verbose,
        }
    }
}

fn parse_line<F: FromStr>() -> Result<F, F::Err> {
    let mut buf = String::new();
    io::stdin().lock().read_line(&mut buf).expect("error reading stdin");

    buf.trim_end_matches('\n').parse()
}

impl World for ConsoleWorld {
    fn get(&mut self) -> Result<MemoryValue, &'static str> {
        print!("> "); io::stdout().flush().unwrap();
        parse_line().map_err(|_| INVALID_NUMBER_FORMAT_ERROR)
    }

    fn put(&mut self, val: &MemoryValue) {
        println!("{}", val);
    }

    fn log(&mut self, message: &str) {
        if self.verbose {
            eprintln!("{}", message);
        }
    }
}

pub struct MemoryWorld {
    inputs: Vec<MemoryValue>,
    outputs: Vec<MemoryValue>,
    logs: Vec<String>,
}

impl MemoryWorld {
    pub fn new(inputs: Vec<MemoryValue>) -> MemoryWorld {
        MemoryWorld {
            inputs,
            outputs: vec![],
            logs: vec![],
        }
    }

    pub fn output(&self) -> String {
        let mut buf = String::with_capacity(self.outputs.len() * 8);
        for val in &self.outputs {
            writeln!(&mut buf, "{}", val).expect("writing to string failed");
        }

        buf
    }
}

impl World for MemoryWorld {
    fn get(&mut self) -> Result<MemoryValue, &'static str> {
        if let Some(val) = self.inputs.pop() {
            Ok(val)
        } else {
            Err("empty world input")
        }
    }

    fn put(&mut self, val: &MemoryValue) {
        self.outputs.push(val.clone());
    }

    fn log(&mut self, message: &str) {
        self.logs.push(message.to_owned());
    }
}

pub struct Interpreter {
    world: Rc<RefCell<dyn World>>,
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
    pub fn new(world: Rc<RefCell<dyn World>>, program: Vec<Instruction>) -> Interpreter {
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
        self.memory.get(&index).ok_or(UNINITIALIZED_ERROR)
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
        self.memory.insert(index, new_value.clone());

        Ok(())
    }

    fn mutate_bin<F: Fn(&MemoryValue, &MemoryValue) -> MemoryValue>(&mut self, index: i64, value_index: i64, f: F) -> IResult {
        let acc_value = self.get_initialized(index)?;
        let value = self.get_initialized(value_index)?;
        let new_value = f(acc_value, value);
        self.log(|| format!("mutate_bin: [{}] <- f({}, {})", index, acc_value, value));
        self.memory.insert(index, new_value.clone());

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
                    self.log(|| format!("INC"));
                    self.mutate(0, |a| a + 1)?;
                    self.instr_ptr += 1;
                },
                Instruction::Dec => {
                    self.cost += 1;
                    self.log(|| format!("DEC"));
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
