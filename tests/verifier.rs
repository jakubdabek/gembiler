use ::gembiler::verifier::{SemanticVerifier, Error, verify};
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

    let result = verify(&program);

    assert_eq!(result, Ok(()));
}

#[test]
fn no_declarations_err_undeclared() {
    let program = Program {
        declarations: None,
        commands: vec![
            Command::Read {
                target: Identifier::VarAccess {
                    name: String::from("a"),
                },
            },
        ],
    };

    let result = verify(&program);
    let expected_errors = vec![
        Error::UndeclaredVariable { name: String::from("a") },
    ];

    assert_eq!(result, Err(expected_errors));
}

#[test]
fn no_declarations_err_undeclared_all() {
    let program = Program {
        declarations: None,
        commands: vec![
            Command::Read {
                target: Identifier::VarAccess {
                    name: String::from("a"),
                },
            },
            Command::Read {
                target: Identifier::VarAccess {
                    name: String::from("b"),
                },
            },
        ],
    };

    let result = verify(&program);
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

    let result = verify(&program);

    assert_eq!(result, Ok(()));
}

#[test]
fn no_declarations_for_err() {
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
                        value: Value::Num(1),
                    }
                ],
            },
            Command::Write {
                value: Value::Identifier(Identifier::VarAccess {
                    name: String::from("i"),
                }),
            }
        ],
    };

    let result = verify(&program);

    let expected_errors = vec![
        Error::UndeclaredVariable { name: String::from("i") },
    ];

    assert_eq!(result, Err(expected_errors));
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

    let result = verify(&program);

    assert_eq!(result, Ok(()));
}

#[test]
fn simple_declarations_ok() {
    let program = Program {
        declarations: Some(vec![
            Declaration::Var { name: String::from("a") },
            Declaration::Var { name: String::from("b") },
        ]),
        commands: vec![
            Command::Read {
                target: Identifier::VarAccess {
                    name: String::from("a"),
                },
            },
            Command::Read {
                target: Identifier::VarAccess {
                    name: String::from("b"),
                },
            },
        ],
    };

    let result = verify(&program);

    assert_eq!(result, Ok(()));
}

#[test]
fn simple_declarations_err() {
    let program = Program {
        declarations: Some(vec![
            Declaration::Var { name: String::from("a") },
        ]),
        commands: vec![
            Command::Read {
                target: Identifier::VarAccess {
                    name: String::from("a"),
                },
            },
            Command::Write {
                value: Value::Identifier(Identifier::VarAccess {
                    name: String::from("a"),
                }),
            },
            Command::Read {
                target: Identifier::VarAccess {
                    name: String::from("b"),
                },
            },
        ],
    };

    let result = verify(&program);
    let expected_errors = vec![
        Error::UndeclaredVariable { name: String::from("b") },
    ];

    assert_eq!(result, Err(expected_errors));
}

#[test]
fn arr_declarations_ok() {
    let program = Program {
        declarations: Some(vec![
            Declaration::Var { name: String::from("a") },
            Declaration::Array { name: String::from("arr"), start: 0, end: 10 },
        ]),
        commands: vec![
            Command::Read {
                target: Identifier::VarAccess {
                    name: String::from("a"),
                },
            },
            Command::Read {
                target: Identifier::ArrAccess {
                    name: String::from("arr"),
                    index: String::from("a"),
                },
            },
            Command::Write {
                value: Value::Identifier(Identifier::ArrAccess {
                    name: String::from("arr"),
                    index: String::from("a"),
                }),
            },
        ],
    };

    let result = verify(&program);

    assert_eq!(result, Ok(()));
}

#[test]
fn arr_declarations_err() {
    let program = Program {
        declarations: None,
        commands: vec![
            Command::Read {
                target: Identifier::ArrConstAccess {
                    name: String::from("arr"),
                    index: 0,
                },
            },
            Command::Write {
                value: Value::Identifier(Identifier::ArrAccess {
                    name: String::from("arr"),
                    index: String::from("a"),
                }),
            },
        ],
    };

    let result = verify(&program);
    let expected_errors = vec![
        Error::UndeclaredVariable { name: String::from("arr") },
        Error::UndeclaredVariable { name: String::from("arr") },
        Error::UndeclaredVariable { name: String::from("a") },
    ];

    assert_eq!(result, Err(expected_errors));
}

#[test]
fn no_declarations_for_modification_err() {
    let program = Program {
        declarations: None,
        commands: vec![
            Command::For {
                counter: "i".to_string(),
                ascending: false,
                from: Value::Num(1),
                to: Value::Num(10),
                commands: vec![
                    Command::Read {
                        target: Identifier::VarAccess {
                            name: String::from("i"),
                        }
                    },
                ],
            },
            Command::Write {
                value: Value::Identifier(Identifier::VarAccess {
                    name: String::from("i"),
                }),
            }
        ],
    };

    let result = verify(&program);

    let expected_errors = vec![
        Error::ForCounterModification { name: String::from("i") },
        Error::UndeclaredVariable { name: String::from("i") },
    ];

    assert_eq!(result, Err(expected_errors));
}

#[test]
fn no_declarations_nested_for_modification_err() {
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
                            Command::Read {
                                target: Identifier::VarAccess {
                                    name: String::from("i"),
                                },
                            },
                            Command::Read {
                                target: Identifier::VarAccess {
                                    name: String::from("j"),
                                },
                            }
                        ],
                    },
                ],
            },
        ],
    };

    let result = verify(&program);

    let expected_errors = vec![
        Error::ForCounterModification { name: String::from("i") },
        Error::ForCounterModification { name: String::from("j") },
    ];

    assert_eq!(result, Err(expected_errors));
}

#[test]
fn for_complex_err() {
    let program = Program {
        declarations: Some(vec![
            Declaration::Array { name: String::from("arr"), start: 0, end: 10, }
        ]),
        commands: vec![
            Command::For {
                counter: "i".to_string(),
                ascending: false,
                from: Value::Num(1),
                to: Value::Num(10),
                commands: vec![
                    Command::Read {
                        target: Identifier::VarAccess {
                            name: String::from("i"),
                        }
                    },
                    Command::Assign {
                        target: Identifier::VarAccess {
                            name: String::from("i"),
                        },
                        expr: Expression::Compound {
                            left: Value::Identifier(Identifier::ArrAccess {
                                name: String::from("arr"),
                                index: String::from("a"),
                            }),
                            op: ExprOp::Plus,
                            right: Value::Num(1),
                        },
                    },
                ],
            },
            Command::Write {
                value: Value::Identifier(Identifier::VarAccess {
                    name: String::from("i"),
                }),
            }
        ],
    };

    let result = verify(&program);

    let expected_errors = vec![
        Error::ForCounterModification { name: String::from("i") },
        Error::ForCounterModification { name: String::from("i") },
        Error::UndeclaredVariable { name: String::from("a") },
        Error::UndeclaredVariable { name: String::from("i") },
    ];

    assert_eq!(result, Err(expected_errors));
}
