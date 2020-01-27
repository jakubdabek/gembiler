use std::fmt;
use std::fmt::Display;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    Get,
    Put,
    Load(u64),
    Loadi(u64),
    Store(u64),
    Storei(u64),
    Add(u64),
    Sub(u64),
    Shift(u64),
    Mul(u64),
    Div(u64),
    Mod(u64),
    Inc,
    Dec,
    Jump(u64),
    Jpos(u64),
    Jzero(u64),
    Jneg(u64),
    Halt,
}

pub struct InstructionListPrinter<'a>(pub &'a [Instruction]);

impl Display for InstructionListPrinter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for instruction in self.0 {
            match instruction {
                Instruction::Get => writeln!(f, "GET")?,
                Instruction::Put => writeln!(f, "PUT")?,
                Instruction::Load(arg) => writeln!(f, "LOAD {}", arg)?,
                Instruction::Loadi(arg) => writeln!(f, "LOADI {}", arg)?,
                Instruction::Store(arg) => writeln!(f, "STORE {}", arg)?,
                Instruction::Storei(arg) => writeln!(f, "STOREI {}", arg)?,
                Instruction::Add(arg) => writeln!(f, "ADD {}", arg)?,
                Instruction::Sub(arg) => writeln!(f, "SUB {}", arg)?,
                Instruction::Shift(arg) => writeln!(f, "SHIFT {}", arg)?,
                Instruction::Mul(arg) => writeln!(f, "MUL {}", arg)?,
                Instruction::Div(arg) => writeln!(f, "DIV {}", arg)?,
                Instruction::Mod(arg) => writeln!(f, "MOD {}", arg)?,
                Instruction::Inc => writeln!(f, "INC")?,
                Instruction::Dec => writeln!(f, "DEC")?,
                Instruction::Jump(arg) => writeln!(f, "JUMP {}", arg)?,
                Instruction::Jpos(arg) => writeln!(f, "JPOS {}", arg)?,
                Instruction::Jzero(arg) => writeln!(f, "JZERO {}", arg)?,
                Instruction::Jneg(arg) => writeln!(f, "JNEG {}", arg)?,
                Instruction::Halt => writeln!(f, "HALT")?,
            }
        }

        Ok(())
    }
}

impl Instruction {
    pub fn cost(&self) -> u64 {
        use Instruction::*;
        match self {
            Get => 100,
            Put => 100,
            Load(_) => 10,
            Loadi(_) => 20,
            Store(_) => 10,
            Storei(_) => 20,
            Add(_) => 10,
            Sub(_) => 10,
            Shift(_) => 5,
            Mul(_) => 50,
            Div(_) => 50,
            Mod(_) => 50,
            Inc => 1,
            Dec => 1,
            Jump(_) => 1,
            Jpos(_) => 1,
            Jzero(_) => 1,
            Jneg(_) => 1,
            Halt => 0,
        }
    }
}
