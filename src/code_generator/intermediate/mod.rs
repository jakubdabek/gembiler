use parser::ast;
use parser::ast::visitor::Visitable;

mod variable;
pub use variable::*;
use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Constant(pub i64);

impl Constant {
    pub fn repr(&self) -> String {
        format!("const({})", self.0)
    }

    pub fn value(&self) -> i64 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub enum Access {
    Constant(Constant),
    Variable(VariableIndex),
    ArrayStatic(VariableIndex, Constant),
    ArrayDynamic(VariableIndex, VariableIndex),
}

#[derive(Debug)]
pub enum Instruction {
    Label { label: Label },

    Load { access: Access },

    PreStore { access: Access },
    Store { access: Access },

//    LoadDirect { variable: VariableIndex }, // p0 <- var
//    LoadIndirect { variable: VariableIndex }, // p0 <- [var]
//
//    StoreDirect { variable: VariableIndex }, // var <- p0
//    StoreIndirect { variable: VariableIndex }, // [var] <- p0

    Operation { op: ast::ExprOp, operand: VariableIndex }, // p0 <- p0 <op> operand

    Jump { label: Label },
    JNegative { label: Label },
    JPositive { label: Label },
    JZero { label: Label },

    Get, // print p0
    Put, // read p0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Label {
    id: usize,
}

impl Label {
    fn new(id: usize) -> Self {
        Label {
            id,
        }
    }
}

pub struct Context {
    variables: Vec<UniqueVariable>,
    constants: BTreeMap<Constant, VariableIndex>,
    labels: Vec<Label>,
    instructions: Vec<Instruction>,
}

impl Context {
    pub fn variables(&self) -> &[UniqueVariable] {
        self.variables.as_slice()
    }

    pub fn constants(&self) -> &BTreeMap<Constant, VariableIndex> {
        &self.constants
    }

    pub fn labels(&self) -> &[Label] {
        self.labels.as_slice()
    }

    pub fn instructions(&self) -> &[Instruction] {
        self.instructions.as_slice()
    }
}

impl Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result{
        writeln!(f, "Context {{\n  variables: [")?;
        for var in &self.variables {
            writeln!(f, "    {:?},", var)?;
        }
        write!(f, "  ],\n  constants:")?;
        writeln!(f, " {:?},", self.constants)?;
        write!(f, "  labels:")?;
        writeln!(f, " {:?},", self.labels)?;
        writeln!(f, "  instructions: [")?;
        for instr in &self.instructions {
            writeln!(f, "    {:?},", instr)?;
        }
        write!(f, "  ]\n}}")
    }
}

impl Context {
    fn new() -> Self {
        let mut context = Context {
            variables: vec![],
            constants: BTreeMap::new(),
            labels: vec![],
            instructions: vec![],
        };

        context.add_variable(Variable::Unit {
            name: "p0".to_string(),
        });

        context.add_variable(Variable::Unit {
            name: "tmp$op".to_string(),
        });

        context.add_variable(Variable::Unit {
            name: "tmp$1".to_string(),
        });

        context.add_variable(Variable::Unit {
            name: "tmp$2".to_string(),
        });

        context
    }

    pub fn add_variable(&mut self, variable: Variable) -> VariableIndex {
        let id = VariableIndex::new(self.variables.len());
        self.variables.push(UniqueVariable::new(id, variable));
        id
    }

    pub fn find_variable_by_name(&self, name: &str) -> Option<&UniqueVariable> {
        self.variables.iter().find(|&var| var.variable().name() == name)
    }

    pub fn get_variable(&self, index: &VariableIndex) -> &UniqueVariable {
        self.variables.get(index.value()).expect("nonexistent variable requested")
    }

    pub fn register_constant(&mut self, constant: Constant) -> VariableIndex {
        if self.constants.contains_key(&constant) {
            self.constants[&constant]
        } else {
            let index = self.add_variable(Variable::Unit { name: constant.repr() });
            self.constants.insert(constant, index);
            index
        }
    }

    pub fn get_constant_index(&self, constant: &Constant) -> VariableIndex {
        self.constants[constant]
    }
}

#[derive(Debug)]
struct AccessStack(Vec<Access>);

impl AccessStack {
    fn new() -> Self {
        AccessStack(vec![])
    }

    fn pop(&mut self) -> Access {
        self.0.pop().expect("access stack empty")
    }

    fn peek(&self, index: usize) -> &Access {
        self.0.get(self.0.len() - index - 1).expect("access stack empty")
    }
}

#[derive(Debug)]
struct CodeGenerator {
    context: Context,
    locals: Vec<VariableIndex>,
    access_stack: AccessStack,
}

impl CodeGenerator {
    fn new() -> Self {
        CodeGenerator {
            context: Context::new(),
            locals: vec![],
            access_stack: AccessStack::new(),
        }
    }

    fn add_global(&mut self, variable: Variable) {
        self.context.add_variable(variable);
    }

    fn add_local(&mut self, variable: Variable) -> VariableIndex {
        let var = self.context.add_variable(variable);
        self.locals.push(var);
        var
    }

    fn pop_local(&mut self, index: VariableIndex) {
        let popped = self.locals.pop().expect("locals empty");
        if popped != index {
            panic!("locals incorrectly nested")
        }
    }

    fn find_variable_by_name(&self, name: &str) -> Option<&UniqueVariable> {
        self.locals.iter()
            .map(|ind| self.context.get_variable(ind))
            .rfind(|&var| var.variable().name() == name)
            .or_else(|| self.context.find_variable_by_name(name))
    }

    fn push_access(&mut self, access: Access) {
        self.access_stack.0.push(access);
    }

    fn emit(&mut self, instruction: Instruction) {
        self.context.instructions.push(instruction)
    }

    fn emit_load_visited(&mut self) {
        let access = self.access_stack.pop();
        self.emit(Instruction::Load { access });
    }

    fn emit_pre_store_visited(&mut self) {
        let access = self.access_stack.peek(0).clone();
        self.emit(Instruction::PreStore { access });
    }

    fn emit_store_visited(&mut self) {
        let access = self.access_stack.pop();
        self.emit(Instruction::Store { access });
    }

    fn emit_temporary_store(&mut self) -> VariableIndex {
        let index = self.find_variable_by_name("tmp$op").unwrap().id();
        let access = Access::Variable(index);
        self.emit(Instruction::Store { access });
        index
    }

    fn new_label(&mut self) -> Label {
        let id = self.context.labels.len();
        let label = Label::new(id);
        self.context.labels.push(label.clone());
        label
    }
}

mod visitor_impl;

pub fn generate(program: &ast::Program) -> Result<Context, ()> {
    let mut generator = CodeGenerator::new();
    program.accept(&mut generator);

    Ok(generator.context)
}

#[cfg(test)]
mod test {
    use parser::ast;

    #[test]
    fn it_works() {
        let var_a = ast::Identifier::VarAccess { name: String::from("a") };
        let program = ast::Program {
            declarations: Some(vec![
                ast::Declaration::Var { name: String::from("a") },
            ]),
            commands: vec![
                ast::Command::Read {
                    target: var_a.clone(),
                },
                ast::Command::Write {
                    value: ast::Value::Num(1),
                },
                ast::Command::Write {
                    value: ast::Value::Identifier(var_a.clone()),
                },
            ],
        };

        let result = super::generate(&program);
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}

