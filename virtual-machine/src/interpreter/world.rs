use std::str::FromStr;
use std::io;
use std::io::{Write as _, BufRead as _};
use std::marker::PhantomData;
use std::fmt::{self, Display};
use std::cell::RefCell;
use std::rc::Rc;

const INVALID_NUMBER_FORMAT: &str = "invalid number format";

pub trait World<T> {
    fn get(&mut self) -> Result<T, &'static str>;
    fn put(&mut self, val: &T);
    fn log(&mut self, message: fmt::Arguments);
}

pub fn upcast<T, W: World<T> + 'static>(world: Rc<RefCell<W>>) -> Rc<RefCell<dyn World<T>>> {
    world
}

pub struct ConsoleWorld<T> {
    verbose: bool,
    phantom: PhantomData<T>,
}

impl <T> ConsoleWorld<T> {
    pub fn new(verbose: bool) -> ConsoleWorld<T> {
        ConsoleWorld {
            verbose,
            phantom: PhantomData,
        }
    }
}

fn parse_line<F: FromStr>() -> Result<F, F::Err> {
    let mut buf = String::new();
    io::stdin().lock().read_line(&mut buf).expect("error reading stdin");

    buf.trim_end_matches('\n').parse()
}

impl <T: FromStr + Display> World<T> for ConsoleWorld<T> {
    fn get(&mut self) -> Result<T, &'static str> {
        print!("> "); io::stdout().flush().unwrap();
        parse_line().map_err(|_| INVALID_NUMBER_FORMAT)
    }

    fn put(&mut self, val: &T) {
        println!("{}", val);
    }

    fn log(&mut self, message: fmt::Arguments) {
        if self.verbose {
            eprintln!("{}", message);
        }
    }
}

pub struct MemoryWorld<T> {
    inputs: Vec<T>,
    outputs: Vec<T>,
    logs: Vec<String>,
}

impl <T> MemoryWorld<T> {
    pub fn new(inputs: Vec<T>) -> MemoryWorld<T> {
        MemoryWorld {
            inputs,
            outputs: vec![],
            logs: vec![],
        }
    }

    pub fn output(&self) -> &[T] {
        &self.outputs
    }

    pub fn logs(&self) -> impl Iterator<Item=&str> {
        self.logs.iter().map(|s| s.as_str())
    }
}

impl <T: Clone> World<T> for MemoryWorld<T> {
    fn get(&mut self) -> Result<T, &'static str> {
        if let Some(val) = self.inputs.pop() {
            Ok(val)
        } else {
            Err("empty world input")
        }
    }

    fn put(&mut self, val: &T) {
        self.outputs.push(val.clone());
    }

    fn log(&mut self, message: fmt::Arguments) {
        self.logs.push(message.to_string());
    }
}
