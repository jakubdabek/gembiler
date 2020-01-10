use parser::ast::*;
use parser::ast::visitor::{Visitor, Visitable};

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

pub fn verify(program: Program) -> Result<(), Vec<Error>>{
    let mut verifier = SemanticVerifier::new();
    let result = program.accept(&mut verifier);
    result
}

pub fn add_errors(current: Result<(), Vec<Error>>, new: Result<(), Vec<Error>>) -> Result<(), Vec<Error>> {
    if let Err(mut errors) = current {
        if let Err(new_errors) = new {
            errors.extend(new_errors.into_iter())
        }
        Err(errors)
    } else {
        new
    }
}

fn collect_errors<I: IntoIterator<Item=Result<(), Vec<Error>>>>(collection: I) -> Result<(), Vec<Error>> {
    collection.into_iter().fold(Ok(()), |acc, res| add_errors(acc, res))
}

impl Visitor for SemanticVerifier {
    type Err = Vec<Error>;

    fn visit_declarations(&mut self, declarations: &Declarations) -> Result<(), Self::Err> {
        declarations.iter()
            .map(|declaration| self.visit(declaration))
            .fold(Ok(()), |acc, res| {
                add_errors(acc, res)
            })?;

        self.globals = declarations.clone();
        Ok(())
    }

    fn visit_declaration(&mut self, declaration: &Declaration) -> Result<(), Self::Err> {
        match declaration {
            Declaration::Var { .. } => Ok(()),
            Declaration::Array { name, start, end } => {
                if start > end {
                    Err(vec![Error::InvalidArrayRange {
                        name: name.clone(),
                        start: *start,
                        end: *end,
                    }])
                } else {
                    Ok(())
                }
            }
        }
    }

    fn visit_for_command(&mut self, counter: &str, _ascending: bool, from: &Value, to: &Value, commands: &Commands) -> Result<(), Self::Err> {
        self.visit(from)?;
        self.visit(to)?;
        self.locals.push(counter.to_string());
        let result = self.visit_commands(commands);
        self.locals.pop();
        result
    }

    fn visit_read_command(&mut self, target: &Identifier) -> Result<(), Self::Err> {
        Ok(()) // TODO
    }

    fn visit_assign_command(&mut self, target: &Identifier, expr: &Expression) -> Result<(), Self::Err> {
        Ok(()) // TODO
    }

    fn visit_num_value(&mut self, num: i64) -> Result<(), Self::Err> {
        Ok(())
    }

    fn visit_identifier(&mut self, identifier: &Identifier) -> Result<(), Self::Err> {
        identifier.names().iter().map(|&name| {
            self.locals.iter()
                .find(|&local| name == local)
                .map(|_| ())
                .or_else(|| {
                    self.globals
                        .iter()
                        .find(|&global| global.name() == name)
                        .map(|_| ())
                })
                .ok_or(name)
        }).fold(Ok(()), |mut acc, res| {
            if let Err(name) = res {
                if let Ok(_) = acc {
                    acc = Err(vec![]);
                }
                let mut tmp = acc.unwrap_err();
                tmp.push(Error::UndeclaredVariable { name: name.to_owned() });
                acc = Err(tmp);
            }
            acc
        })
    }
}
