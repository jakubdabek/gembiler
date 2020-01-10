#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub declarations: Option<Declarations>,
    pub commands: Commands,
}

pub type Declarations = Vec<Declaration>;

#[derive(Debug, PartialEq, Clone)]
pub enum Declaration {
    Var { name: String },
    Array { name: String, start: i64, end: i64 },
}

impl Declaration {
    pub fn name(&self) -> &str {
        use Declaration::*;
        match self {
            Var { name } => name,
            Array { name, .. } => name,
        }
    }
}

pub type Commands = Vec<Command>;

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    IfElse { condition: Condition, positive: Commands, negative: Commands },
    If { condition: Condition, positive: Commands },
    While { condition: Condition, commands: Commands },
    Do { commands: Commands, condition: Condition },
    For { counter: String, ascending: bool, from: Value, to: Value, commands: Commands },
    Read { target: Identifier },
    Write { value: Value },
    Assign { target: Identifier, expr: Expression },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ExprOp { Plus, Minus, Times, Div, Mod, }

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Simple { value: Value },
    Compound { left: Value, op: ExprOp, right: Value },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RelOp { EQ, NEQ, LEQ, LE, GEQ, GE, }

#[derive(Debug, PartialEq, Clone)]
pub struct Condition {
    pub left: Value,
    pub op: RelOp,
    pub right: Value,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Num(i64),
    Identifier(Identifier)
}

#[derive(Debug, PartialEq, Clone)]
pub enum Identifier {
    VarAccess { name: String },
    ArrAccess { name: String, index: String },
    ArrConstAccess { name: String, index: i64 },
}

impl Identifier {
    pub fn names(&self) -> Vec<&str> {
        match self {
            Identifier::VarAccess { name } => vec![name],
            Identifier::ArrAccess { name, index } => vec![name, index],
            Identifier::ArrConstAccess { name, .. } => vec![name],
        }
    }
}
