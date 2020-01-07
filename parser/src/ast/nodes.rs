#[derive(Debug, PartialEq)]
pub struct Program {
    pub declarations: Option<Declarations>,
    pub commands: Commands,
}

type Declarations = Vec<Declaration>;

#[derive(Debug, PartialEq)]
pub enum Declaration {
    Var { name: String },
    Array { name: String, start: i32, end: i32 },
}

type Commands = Vec<Command>;

#[derive(Debug, PartialEq)]
pub enum Command {
    IfElse { condition: Condition, positive: Commands, negative: Commands },
    If { condition: Condition, positive: Commands },
    While { condition: Condition, commands: Commands },
    Do { commands: Commands, condition: Condition },
    For { counter: Identifier, ascending: bool, from: Value, to: Value },
    Read { target: Identifier },
    Write { value: Value },
    Assign { target: Identifier, expr: Expression },
}

#[derive(Debug, PartialEq, Eq)]
pub enum ExprOp { Plus, Minus, Times, Div, Mod, }

#[derive(Debug, PartialEq)]
pub enum Expression {
    Simple { value: Value },
    Compound { left: Value, op: ExprOp, right: Value },
}

#[derive(Debug, PartialEq, Eq)]
pub enum RelOp { EQ, NEQ, LEQ, LE, GEQ, GE, }

#[derive(Debug, PartialEq)]
pub struct Condition {
    pub left: Value,
    pub op: RelOp,
    pub right: Value,
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Num(i32),
    Identifier(Identifier)
}

#[derive(Debug, PartialEq)]
pub enum Identifier {
    VarAccess { name: String },
    ArrAccess { name: String, index: String },
    ArrConstAccess { name: String, index: i64 },
}

