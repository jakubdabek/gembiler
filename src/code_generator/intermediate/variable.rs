#[derive(Debug)]
pub enum Variable {
    Unit { name: String },
    Array { name: String, start: i64, end: i64 },
}

impl Variable {
    fn size(&self) -> usize {
        use Variable::*;
        match self {
            Unit { .. } => 1,
            Array { start, end, .. } => (end - start) as usize,
        }
    }

    fn name(&self) -> &str {
        use Variable::*;
        match self {
            Unit { name } => name,
            Array { name, .. } => name,
        }
    }
}

#[derive(Debug)]
pub struct UniqueVariable {
    id: VariableIndex,
    variable: Variable,
}

impl UniqueVariable {
    pub fn new(id: VariableIndex, variable: Variable) -> Self {
        UniqueVariable {
            id,
            variable,
        }
    }

    pub fn id(&self) -> VariableIndex {
        self.id
    }

    pub fn name(&self) -> &str {
        self.variable.name()
    }
}

impl PartialEq for UniqueVariable {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for UniqueVariable {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VariableIndex {
    id: usize,
}

impl VariableIndex {
    pub fn new(id: usize) -> Self {
        VariableIndex {
            id,
        }
    }

    pub fn value(&self) -> usize {
        self.id
    }
}

pub enum VariableValue {
    Unit { }
}
