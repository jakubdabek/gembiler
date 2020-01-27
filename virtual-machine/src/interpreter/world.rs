use std::cell::RefCell;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::io::{BufRead as _, Write as _};
use std::marker::PhantomData;
use std::rc::Rc;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InvalidInput,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        use Error::*;
        match self {
            InvalidInput => write!(f, "invalid input"),
        }
    }
}

pub trait World<T> {
    fn get(&mut self) -> Result<T, Error>;
    fn put(&mut self, val: &T);
    fn log(&mut self, message: fmt::Arguments);
}

pub fn upcast<T, W: World<T> + 'static>(world: Rc<RefCell<W>>) -> Rc<RefCell<dyn World<T>>> {
    world
}

#[derive(Debug)]
pub struct ConsoleWorld<T> {
    verbose: bool,
    phantom: PhantomData<T>,
}

impl<T> ConsoleWorld<T> {
    pub fn new(verbose: bool) -> ConsoleWorld<T> {
        ConsoleWorld {
            verbose,
            phantom: PhantomData,
        }
    }
}

fn parse_line<F: FromStr>() -> Result<F, F::Err> {
    let mut buf = String::new();
    io::stdin()
        .lock()
        .read_line(&mut buf)
        .expect("error reading stdin");

    buf.trim_matches(&[' ', '\t', '\n', '\r'][..]).parse()
}

impl<T: FromStr + Display> World<T> for ConsoleWorld<T> {
    fn get(&mut self) -> Result<T, Error> {
        print!("? ");
        io::stdout().flush().unwrap();
        parse_line().map_err(|_| Error::InvalidInput)
    }

    fn put(&mut self, val: &T) {
        println!("> {}", val);
    }

    fn log(&mut self, message: fmt::Arguments) {
        if self.verbose {
            eprintln!("{}", message);
        }
    }
}

#[derive(Debug)]
pub struct MemoryWorld<T> {
    inputs: Vec<T>,
    outputs: Vec<T>,
    logs: Vec<String>,
}

impl<T> MemoryWorld<T> {
    pub fn new(mut inputs: Vec<T>) -> MemoryWorld<T> {
        inputs.reverse();
        MemoryWorld {
            inputs,
            outputs: vec![],
            logs: vec![],
        }
    }

    pub fn output(&self) -> &[T] {
        &self.outputs
    }

    pub fn logs(&self) -> impl Iterator<Item = &str> {
        self.logs.iter().map(|s| s.as_str())
    }
}

impl<T: Clone> World<T> for MemoryWorld<T> {
    fn get(&mut self) -> Result<T, Error> {
        if let Some(val) = self.inputs.pop() {
            Ok(val)
        } else {
            Err(Error::InvalidInput)
        }
    }

    fn put(&mut self, val: &T) {
        self.outputs.push(val.clone());
    }

    fn log(&mut self, message: fmt::Arguments) {
        self.logs.push(message.to_string());
    }
}
