use std::cell::RefCell;
use std::{fs, env, io};
use std::rc::Rc;

mod instruction;
mod interpreter;
mod parser;

use crate::interpreter::{world, Interpreter};
use std::path::Path;

#[derive(Debug)]
enum Error {
    FsError(io::Error),
    ParseError(String),
    InterpretError(interpreter::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::FsError(e)
    }
}

impl From<parser::Error> for Error {
    fn from(e: parser::Error) -> Self {
        Error::ParseError(e.to_string())
    }
}

impl From<interpreter::Error> for Error {
    fn from(e: interpreter::Error) -> Self {
        Error::InterpretError(e)
    }
}

fn interpret<P: AsRef<Path>>(path: P, verbose: bool) -> Result<u64, Error> {
    let text = fs::read_to_string(path)?;

    let world = Rc::new(RefCell::new(world::ConsoleWorld::new(verbose)));
    let program = parser::create_program(&text)?;
    let mut interpreter = Interpreter::new(world::upcast(Rc::clone(&world)), program);
    Ok(interpreter.interpret()?)
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let len = args.len();

    match len {
        _ if len < 2 => println!("Usage: {} <input>", args[0]),
        _ => {
            let result = interpret(args[1].as_str(), args.get(2).map_or(false, |v| v == "-v"));
            match result {
                Ok(cost) => println!("Program successful (cost: {})", cost),
                Err(error) => {
                    match error {
                        Error::FsError(e) => {
                            println!("Error while reading file: {}", e);
                        },
                        Error::ParseError(e) => {
                            println!("Error while parsing file: {}", e);
                        },
                        Error::InterpretError(e) => {
                            println!("Error while running: {}", e);
                        },
                    }
                },
            }
        },
    }
}
