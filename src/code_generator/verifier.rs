use parser::ast::*;
use parser::ast::visitor::{Visitor, Visitable, ResultCombineErr, VisitorResult, VisitorResultVec};

#[derive(Debug)]
pub struct SemanticVerifier {
    globals: Vec<Declaration>,
    locals: Vec<String>,
}

impl SemanticVerifier {
    pub fn new() -> SemanticVerifier {
        SemanticVerifier {
            globals: vec![],
            locals: vec![],
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidArrayRange {
        name: String,
        start: i64,
        end: i64
    },
    UndeclaredVariable { name: String },
}

pub fn verify(program: Program) -> Result<(), Vec<Error>> {
    let mut verifier = SemanticVerifier::new();
    let result = program.accept(&mut verifier);
    result.into_result().map_err(|v| v.into())
}



impl Visitor for SemanticVerifier {
    type Result = ResultCombineErr<(), VisitorResultVec<Error>>;

    fn visit_declarations(&mut self, declarations: &Declarations) -> Self::Result {
        let results = declarations.iter().map(|declaration| self.visit(declaration));
        let res = Self::Result::combine_collection(results);

        self.globals = declarations.clone();

        res
    }

    fn visit_declaration(&mut self, declaration: &Declaration) -> Self::Result {
        match declaration {
            Declaration::Var { .. } => Self::Result::identity(),
            Declaration::Array { name, start, end } => {
                if start > end {
                    Err(vec![Error::InvalidArrayRange {
                        name: name.clone(),
                        start: *start,
                        end: *end,
                    }].into()).into()
                } else {
                    Self::Result::identity()
                }
            }
        }
    }

    fn visit_for_command(&mut self, counter: &str, _ascending: bool, from: &Value, to: &Value, commands: &Commands) -> Self::Result {
        let result = self.visit(from)
            .combine(self.visit(to));
        self.locals.push(counter.to_string());
        let result = result.combine(self.visit_commands(commands));
        self.locals.pop();
        result
    }

    fn visit_read_command(&mut self, target: &Identifier) -> Self::Result {
        Self::Result::identity() // TODO
    }

    fn visit_assign_command(&mut self, target: &Identifier, expr: &Expression) -> Self::Result {
        Self::Result::identity() // TODO
    }

    fn visit_num_value(&mut self, num: i64) -> Self::Result {
        Self::Result::identity()
    }

    fn visit_identifier(&mut self, identifier: &Identifier) -> Self::Result {
        let results = identifier.names().into_iter().map(|name| {
            self.locals.iter()
                .find(|&local| name == local)
                .map(|_| ())
                .or_else(|| {
                    self.globals
                        .iter()
                        .find(|&global| global.name() == name)
                        .map(|_| ())
                })
                .ok_or(vec![Error::UndeclaredVariable { name: name.to_owned() }].into())
                .into()
        });

        Self::Result::combine_collection(results)
    }
}
