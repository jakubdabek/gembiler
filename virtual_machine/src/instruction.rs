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
    Inc,
    Dec,
    Jump(u64),
    Jpos(u64),
    Jzero(u64),
    Jneg(u64),
    Halt,
}
