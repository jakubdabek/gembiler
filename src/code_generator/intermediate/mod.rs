use parser::ast;
use parser::ast::visitor::Visitable;

mod variable;
use variable::*;
use std::rc::Rc;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Constant(i64);

#[derive(Debug)]
enum Access {
    Constant(Constant),
    Variable(VariableIndex),
    Array,
}

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Label { label: Label },
    Load { variable: VariableIndex },
    IndirectLoad { variable: VariableIndex },
    Operation { op: ast::ExprOp, operand: VariableIndex },
    RegisterStore { },
    Read { target: VariableIndex },
    Write,
}

#[derive(Debug, PartialEq, PartialOrd)]
struct Label {
    id: usize,
}

impl Label {
    fn new(id: usize) -> Self {
        Label {
            id,
        }
    }
}

#[derive(Debug)]
pub struct Context {
    variables: Vec<UniqueVariable>,
    constants: BTreeMap<Constant, VariableIndex>,
    labels: Vec<Label>,
    instructions: Vec<Instruction>,
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

        context
    }

    fn add_variable(&mut self, variable: Variable) -> VariableIndex {
        let id = VariableIndex::new(self.variables.len());
        self.variables.push(UniqueVariable::new(id, variable));
        id
    }

    pub fn find_variable_by_name(&self, name: &str) -> Option<&UniqueVariable> {
        self.variables.iter().find(|&var| var.name() == name)
    }

    pub fn get_variable(&self, index: &VariableIndex) -> &UniqueVariable {
        self.variables.get(index.value()).expect("nonexistent variable requested")
    }

    pub fn get_constant(&mut self, constant: Constant) -> VariableIndex {
        if self.constants.contains_key(&constant) {
            *self.constants[&constant]
        } else {
            let index = self.add_variable(Variable::Unit { name: constant.to_string() });
            self.constants.insert(constant, index);
            index
        }
    }
}

#[derive(Debug)]
struct AccessStack(Vec<Access>);

impl AccessStack {
    fn new() -> Self {
        AccessStack(vec![])
    }

    fn access_constant(&mut self, constant: i64) {
        self.0.push(Access::Constant(Constant(constant)));
    }

    fn access_variable(&mut self, variable: VariableIndex) {
        self.0.push(Access::Variable(variable));
    }

    fn access_array(&mut self) {
        self.0.push(Access::Array);
    }

    fn pop(&mut self) -> Access {
        self.0.pop().expect("access stack empty")
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

    fn find_variable_by_name(&self, name: &str) -> Option<&UniqueVariable> {
        self.locals.iter()
            .map(|ind| self.context.get_variable(ind))
            .find(|&var| var.name() == name)
            .or_else(|| self.context.find_variable_by_name(name))
    }

    fn emit(&mut self, instruction: Instruction) {
        self.context.instructions.push(instruction)
    }

    fn access_variable(&mut self, access: Access) -> VariableIndex {
        match access {
            Access::Constant(constant) => {
                self.context.get_constant(constant)
            },
            Access::Variable(variable) => variable,
            Access::Array => panic!("invalid variable on access stack"),
        }
    }

    fn emit_rvalue(&mut self) {
        let top_access = self.access_stack.pop();
        match top_access {
            Access::Constant(constant) => {
                let constant_index = self.context.get_constant(constant);
                self.emit(Instruction::Load { variable: constant_index });
            },
            Access::Variable(index) => {
                self.emit(Instruction::Load { variable: index });
            },
            Access::Array => {
                let array_access = self.access_stack.pop();
                let index_access = self.access_stack.pop();
                if let Access::Variable(array_index) = array_access {
                    self.emit(Instruction::Load { variable: array_index });
                    let operand = self.access_variable(index_access);
                    self.emit(Instruction::Operation {
                        op: ast::ExprOp::Plus,
                        operand,
                    });
                    self.emit(Instruction::IndirectLoad)
                } else {
                    panic!("invalid access stack")
                }
            },
        }
    }

    fn accesses(&mut self) -> &mut AccessStack {
        &mut self.access_stack
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
        let program = ast::Program {
            declarations: None,
            commands: vec![
                ast::Command::Write {
                    value: ast::Value::Num(1),
                }
            ],
        };

        let result = super::generate(&program);

        assert!(result.is_ok());
    }
}

