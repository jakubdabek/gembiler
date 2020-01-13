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
