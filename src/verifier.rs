use parser::ast::visitor::{ResultCombineErr, Visitable, Visitor, VisitorResult, VisitorResultVec};
use parser::ast::*;
use std::fmt::{self, Display, Formatter};

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

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    InvalidArrayRange { name: String, start: i64, end: i64 },
    UndeclaredVariable { name: String },
    ForCounterModification { name: String },
    InvalidVariableUsage { name: String },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            InvalidArrayRange { name, start, end } => write!(f, "invalid array range: {}({}:{})", name, start, end),
            UndeclaredVariable { name } => write!(f, "undeclared variable {}", name),
            ForCounterModification { name } => write!(f, "illegal modification of for loop counter {}", name),
            InvalidVariableUsage { name } => write!(f, "invalid variable usage: {}", name),
        }
    }
}

pub fn verify(program: Program) -> Result<Program, Vec<Error>> {
    let mut verifier = SemanticVerifier::new();
    let result = program.accept(&mut verifier);
    result.into_result().map(|_| program).map_err(|v| v.into_vec())
}

impl SemanticVerifier {
    fn get_global(&self, name: &str) -> Option<&Declaration> {
        self.globals.iter().find(|&global| global.name() == name)
    }

    fn get_local(&self, name: &str) -> Option<&str> {
        self.locals
            .iter()
            .find(|&local| local == name)
            .map(|s| s.as_str())
    }

    fn check_modification(&self, name: &str) -> Result<(), Error> {
        self.get_local(name)
            .map_or(
                Ok(()),
                |_| {
                    Err(Error::ForCounterModification {
                        name: name.to_owned(),
                    })
                }
            )
    }

    fn check_var_usage(&self, name: &str) -> Result<(), Error> {
        self.get_global(name)
            .map(|g| {
                match g {
                    Declaration::Var { .. } => Ok(()),
                    Declaration::Array { .. } => Err(Error::InvalidVariableUsage { name: name.to_owned() }),
                }
            })
            .unwrap_or(Ok(()))
    }

    fn check_array_usage(&self, name: &str) -> Result<(), Error> {
        self.get_global(name)
            .map(|g| {
                match g {
                    Declaration::Var { .. } => Err(Error::InvalidVariableUsage { name: name.to_owned() }.into()),
                    Declaration::Array { .. } => Ok(()),
                }
            })
            .unwrap_or_else(|| {
                self.get_local(name)
                    .map(|_| Err(Error::InvalidVariableUsage { name: name.to_owned() }.into()))
                    .unwrap_or(Ok(()))
            })
    }

    fn check_identifier_usage(&self, identifier: &Identifier) -> <Self as Visitor>::Result {
        match identifier {
            Identifier::VarAccess { name } => {
                self.check_var_usage(name).map_err(Into::into).into()
            },
            Identifier::ArrAccess { name, index } => {
                let main: ResultCombineErr<_, _> = self.check_array_usage(name).map_err(Into::into).into();
                main.combine(
                    self.get_global(index)
                        .map(|g| {
                            match g {
                                Declaration::Var { .. } => Ok(()),
                                Declaration::Array { .. } => Err(Error::InvalidVariableUsage { name: name.to_owned() }.into()),
                            }
                        })
                        .unwrap_or(Ok(()))
                        .into()
                )
            },
            Identifier::ArrConstAccess { name, index } => {
                self.get_global(name)
                    .map(|g| {
                        match g {
                            Declaration::Var { .. } => Err(Error::InvalidVariableUsage { name: name.to_owned() }.into()),
                            Declaration::Array { start, end, .. } => {
                                if index >= start && index <= end {
                                    Ok(())
                                } else {
                                    Err(Error::InvalidVariableUsage { name: name.to_owned() }.into())
                                }
                            },
                        }
                    })
                    .unwrap_or_else(|| {
                        self.get_local(name)
                            .map(|_| Err(Error::InvalidVariableUsage { name: name.to_owned() }.into()))
                            .unwrap_or(Ok(()))
                    }).into()
            },
        }
    }
}

impl<'a> Visitor for SemanticVerifier {
    type Result = ResultCombineErr<(), VisitorResultVec<Error>>;

    fn visit_declarations(&mut self, declarations: &Declarations) -> Self::Result {
        let results = declarations
            .iter()
            .map(|declaration| self.visit(declaration));
        let res = Self::Result::combine_collection(results);

        self.globals = declarations.clone();

        res
    }

    fn visit_declaration(&mut self, declaration: &Declaration) -> Self::Result {
        match declaration {
            Declaration::Var { .. } => Self::Result::identity(),
            Declaration::Array { name, start, end } => {
                if start > end {
                    Err(Error::InvalidArrayRange {
                        name: name.clone(),
                        start: *start,
                        end: *end,
                    }
                    .into())
                    .into()
                } else {
                    Self::Result::identity()
                }
            }
        }
    }

    fn visit_for_command(
        &mut self,
        counter: &str,
        _ascending: bool,
        from: &Value,
        to: &Value,
        commands: &Commands,
    ) -> Self::Result {
        let result = self.visit(from).combine(self.visit(to));
        self.locals.push(counter.to_string());
        let result = result.combine(self.visit_commands(commands));
        self.locals.pop();
        result
    }

    fn visit_read_command(&mut self, target: &Identifier) -> Self::Result {
        self.visit(target)
            .combine(self.check_modification(target.name()).map_err(Into::into).into())
    }

    fn visit_assign_command(&mut self, target: &Identifier, expr: &Expression) -> Self::Result {
        self.visit(target)
            .combine(self.check_modification(target.name()).map_err(Into::into).into())
            .combine(self.visit(expr))
    }

    fn visit_num_value(&mut self, _: i64) -> Self::Result {
        // nothing to be done - allow anything
        Self::Result::identity()
    }

    fn visit_identifier(&mut self, identifier: &Identifier) -> Self::Result {
        let results = identifier.all_names().into_iter().map(|name| {
            self.get_global(name)
                .map(|_| ())
                .or_else(|| self.get_local(name).map(|_| ()))
                .ok_or(
                    Error::UndeclaredVariable {
                        name: name.to_owned(),
                    }
                    .into(),
                )
                .into()
        });

        let undeclared = Self::Result::combine_collection(results);

        let usage = self.check_identifier_usage(identifier);

        undeclared.combine(usage)
    }
}
