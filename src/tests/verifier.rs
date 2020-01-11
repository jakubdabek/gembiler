use ::gembiler::code_generator::verifier::{SemanticVerifier, Error, verify};
use ::parser::ast::*;

#[test]
fn no_declarations_ok() {
    let program = Program {
        declarations: None,
        commands: vec![
            Command::Write {
                value: Value::Num(1),
            }
        ],
    };

    let result = verify(program);

    assert_eq!(result, Ok(()));
}

#[test]
fn no_declarations_err_undeclared() {
    let program = Program {
        declarations: None,
        commands: vec![
            Command::Write {
                value: Value::Identifier(Identifier::VarAccess {
                    name: String::from("a"),
                }),
            },
        ],
    };

    let result = verify(program);
    let expected_errors = vec![
        Error::UndeclaredVariable { name: String::from("a") },
    ];

    assert_eq!(result, Err(expected_errors));
}

#[test]
fn no_declarations_err_undeclared_first() {
    let program = Program {
        declarations: None,
        commands: vec![
            Command::Write {
                value: Value::Identifier(Identifier::VarAccess {
                    name: String::from("a"),
                }),
            },
            Command::Write {
                value: Value::Identifier(Identifier::VarAccess {
                    name: String::from("b"),
                }),
            },
        ],
    };

    let result = verify(program);
    let expected_errors = vec![
        Error::UndeclaredVariable { name: String::from("a") },
        Error::UndeclaredVariable { name: String::from("b") },
    ];

    assert_eq!(result, Err(expected_errors));
}

#[test]
fn no_declarations_for_ok() {
    let program = Program {
        declarations: None,
        commands: vec![
            Command::For {
                counter: "i".to_string(),
                ascending: false,
                from: Value::Num(1),
                to: Value::Num(10),
                commands: vec![
                    Command::Write {
                        value: Value::Identifier(Identifier::VarAccess {
                            name: String::from("i"),
                        }),
                    }
                ],
            },
        ],
    };

    let result = verify(program);

    assert_eq!(result, Ok(()));
}

#[test]
fn no_declarations_nested_for_ok() {
    let program = Program {
        declarations: None,
        commands: vec![
            Command::For {
                counter: "i".to_string(),
                ascending: false,
                from: Value::Num(1),
                to: Value::Num(10),
                commands: vec![
                    Command::For {
                        counter: "j".to_string(),
                        ascending: false,
                        from: Value::Num(101),
                        to: Value::Num(110),
                        commands: vec![
                            Command::Write {
                                value: Value::Identifier(Identifier::VarAccess {
                                    name: String::from("i"),
                                }),
                            },
                            Command::Write {
                                value: Value::Identifier(Identifier::VarAccess {
                                    name: String::from("j"),
                                }),
                            }
                        ],
                    },
                ],
            },
        ],
    };

    let result = verify(program);

    assert_eq!(result, Ok(()));
}
