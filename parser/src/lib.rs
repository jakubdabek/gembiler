extern crate pest;
#[macro_use]
extern crate pest_derive;

pub mod ast;

use std::fs;
use std::path::Path;
use pest::Parser;
use pest::iterators::Pairs;
use crate::ast::*;

#[derive(Parser)]
#[grammar = "program.pest"]
struct ProgramParser;

type AstResult = Result<ast::Program, String>;

pub fn parse_file<P: AsRef<Path>>(path: P) -> AstResult {
    let program_text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    parse_ast(&program_text)
}

pub fn parse_ast(text: &str) -> AstResult {
    let mut program: Pairs<Rule> = ProgramParser::parse(Rule::program, text).map_err(|e| e.to_string())?;

    program = program.next().unwrap().into_inner().next().unwrap().into_inner();

    let optional_declarations = program.next().unwrap();

    let (declarations, commands) = match optional_declarations.as_rule() {
        Rule::declarations => {
            let pairs = optional_declarations.into_inner();
            (Some(parse_declarations(pairs)), program.next().unwrap())
        },
        Rule::commands => (None, optional_declarations),
        _ => unreachable!(),
    };

    let commands = parse_commands(commands.into_inner());

    Ok(ast::Program { declarations, commands })
}

fn parse_declaration(mut pairs: Pairs<Rule>) -> Declaration {
    let declaration = pairs.next().unwrap();
    match declaration.as_rule() {
        Rule::arr_decl => {
            let mut parts = declaration.into_inner();
            Declaration::Array {
                name: parts.next().unwrap().as_str().to_owned(),
                start: parts.next().unwrap().as_str().parse().unwrap(),
                end: parts.next().unwrap().as_str().parse().unwrap(),
            }
        },
        Rule::var_decl => Declaration::Var {
            name: declaration.into_inner().next().unwrap().as_str().to_owned(),
        },
        _ => unreachable!(),
    }
}

fn parse_declarations(pairs: Pairs<Rule>) -> Declarations {
    pairs.map(|pair| parse_declaration(pair.into_inner())).collect()
}

fn parse_identifier(mut pairs: Pairs<Rule>) -> Identifier {
    let name = pairs.next().unwrap().as_str().to_owned();

    if let Some(index) = pairs.next() {
        match index.as_rule() {
            Rule::pidentifier => Identifier::ArrAccess {
                name,
                index: index.as_str().to_owned(),
            },
            Rule::num => Identifier::ArrConstAccess {
                name,
                index: index.as_str().parse().unwrap(),
            },
            _ => unreachable!(),
        }
    } else {
        Identifier::VarAccess {
            name,
        }
    }
}

fn parse_value(mut pairs: Pairs<Rule>) -> Value {
    let value = pairs.next().unwrap();
    match value.as_rule() {
        Rule::num => Value::Num(value.as_str().parse().unwrap()),
        Rule::identifier => Value::Identifier(parse_identifier(value.into_inner())),
        _ => unreachable!(),
    }
}

fn parse_condition(mut pairs: Pairs<Rule>) -> Condition {
    let left = parse_value(pairs.next().unwrap().into_inner());
    let op = match pairs.next().unwrap().as_str() {
        "EQ" => RelOp::EQ,
        "NEQ" => RelOp::NEQ,
        "LEQ" => RelOp::LEQ,
        "LE" => RelOp::LE,
        "GEQ" => RelOp::GEQ,
        "GE" => RelOp::GE,
        _ => unreachable!(),
    };
    let right = parse_value(pairs.next().unwrap().into_inner());

    Condition {
        left,
        op,
        right,
    }
}

fn parse_expression(mut pairs: Pairs<Rule>) -> Expression {
    let left = parse_value(pairs.next().unwrap().into_inner());

    if let Some(op) = pairs.next() {
        let op = match op.as_str() {
            "PLUS" => ExprOp::Plus,
            "MINUS" => ExprOp::Minus,
            "TIMES" => ExprOp::Times,
            "DIV" => ExprOp::Div,
            "MOD" => ExprOp::Mod,
            _ => unreachable!(),
        };
        let right = parse_value(pairs.next().unwrap().into_inner());

        Expression::Compound {
            left,
            op,
            right,
        }
    } else {
        Expression::Simple {
            value: left,
        }
    }
}

fn parse_ifelse(mut pairs: Pairs<Rule>) -> Command {
    let condition = parse_condition(pairs.next().unwrap().into_inner());
    let positive = parse_commands(pairs.next().unwrap().into_inner());
    let negative = parse_commands(pairs.next().unwrap().into_inner());

    Command::IfElse {
        condition,
        positive,
        negative,
    }
}

fn parse_conditional_command(mut pairs: Pairs<Rule>) -> (Condition, Commands) {
    let condition = parse_condition(pairs.next().unwrap().into_inner());
    let commands = parse_commands(pairs.next().unwrap().into_inner());

    (condition, commands)
}

fn parse_if(pairs: Pairs<Rule>) -> Command {
    let (condition, positive) = parse_conditional_command(pairs);

    Command::If {
        condition,
        positive,
    }
}

fn parse_while(pairs: Pairs<Rule>) -> Command {
    let (condition, commands) = parse_conditional_command(pairs);

    Command::While {
        condition,
        commands,
    }
}

fn parse_do(pairs: Pairs<Rule>) -> Command {
    let (condition, commands) = parse_conditional_command(pairs);

    Command::Do {
        condition,
        commands,
    }
}

fn parse_for(mut pairs: Pairs<Rule>) -> Command {
    let counter = pairs.next().unwrap().as_str().to_owned();
    let from = parse_value(pairs.next().unwrap().into_inner());
    let ascending = match pairs.next().unwrap().as_str() {
        "TO" => true,
        "DOWNTO" => false,
        _ => unreachable!(),
    };
    let to = parse_value(pairs.next().unwrap().into_inner());
    let commands = parse_commands(pairs.next().unwrap().into_inner());

    Command::For {
        counter,
        from,
        ascending,
        to,
        commands,
    }
}

fn parse_read(mut pairs: Pairs<Rule>) -> Command {
    let target = parse_identifier(pairs.next().unwrap().into_inner());

    Command::Read {
        target,
    }
}

fn parse_write(mut pairs: Pairs<Rule>) -> Command {
    let value = parse_value(pairs.next().unwrap().into_inner());

    Command::Write {
        value,
    }
}

fn parse_assign(mut pairs: Pairs<Rule>) -> Command {
    let target = parse_identifier(pairs.next().unwrap().into_inner());
    let expr = parse_expression(pairs.next().unwrap().into_inner());

    Command::Assign {
        target,
        expr,
    }
}

fn parse_command(mut pairs: Pairs<Rule>) -> Command {
    let command = pairs.next().unwrap();
    match command.as_rule() {
        Rule::cmd_ifelse => parse_ifelse(command.into_inner()),
        Rule::cmd_if => parse_if(command.into_inner()),
        Rule::cmd_while => parse_while(command.into_inner()),
        Rule::cmd_do => parse_do(command.into_inner()),
        Rule::cmd_for => parse_for(command.into_inner()),
        Rule::cmd_read => parse_read(command.into_inner()),
        Rule::cmd_write => parse_write(command.into_inner()),
        Rule::cmd_assign => parse_assign(command.into_inner()),
        _ => unreachable!(),
    }
}

fn parse_commands(pairs: Pairs<Rule>) -> Commands {
    pairs.map(|pair| parse_command(pair.into_inner())).collect()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simplest() {
        let text = "BEGIN WRITE 0; END";
        let parsed = parse_ast(text);
        let expected = ast::Program {
            declarations: None,
            commands: vec![
                Command::Write { value: Value::Num(0), },
            ],
        };

        assert_eq!(parsed.unwrap(), expected);
    }

    #[test]
    fn simple_declarations() {
        let text = r#"
            DECLARE a, b, c(1:10)
            BEGIN
                WRITE 0;
            END
        "#;
        let parsed = parse_ast(text);
        let expected = ast::Program {
            declarations: Some(vec![
                Declaration::Var { name: String::from("a") },
                Declaration::Var { name: String::from("b") },
                Declaration::Array {
                    name: String::from("c"),
                    start: 1,
                    end: 10,
                },
            ]),
            commands: vec![
                Command::Write { value: Value::Num(0), },
            ],
        };

        assert_eq!(parsed.unwrap(), expected);
    }

    #[test]
    fn program0() {
        let text = r#"
            [ binary representation ]
            DECLARE
                a, b
            BEGIN
                READ a;
                IF a GEQ 0 THEN
                    WHILE a GE 0 DO
                        b ASSIGN a DIV 2;
                        b ASSIGN 2 TIMES b; [ b := a & ~1 ]
                        IF a GE b THEN
                            WRITE 1;
                        ELSE
                            WRITE 0;
                        ENDIF
                        a ASSIGN a DIV 2;
                    ENDWHILE
                ENDIF
            END
        "#;

        let parsed = parse_ast(text);

        let var_a = Identifier::VarAccess { name: String::from("a") };
        let var_b = Identifier::VarAccess { name: String::from("b") };

        let expected = ast::Program {
            declarations: Some(vec![
                Declaration::Var { name: String::from("a") },
                Declaration::Var { name: String::from("b") },
            ]),
            commands: vec![
                Command::Read { target: var_a.clone() },
                Command::If {
                    condition: Condition {
                        left: Value::Identifier(var_a.clone()),
                        op: RelOp::GEQ,
                        right: Value::Num(0),
                    },
                    positive: vec![
                        Command::While {
                            condition: Condition {
                                left: Value::Identifier(var_a.clone()),
                                op: RelOp::GE,
                                right: Value::Num(0),
                            },
                            commands: vec![
                                Command::Assign {
                                    target: var_b.clone(),
                                    expr: Expression::Compound {
                                        left: Value::Identifier(var_a.clone()),
                                        op: ExprOp::Div,
                                        right: Value::Num(2),
                                    }
                                },
                                Command::Assign {
                                    target: var_b.clone(),
                                    expr: Expression::Compound {
                                        left: Value::Num(2),
                                        op: ExprOp::Times,
                                        right: Value::Identifier(var_b.clone()),
                                    }
                                },
                                Command::IfElse {
                                    condition: Condition {
                                        left: Value::Identifier(var_a.clone()),
                                        op: RelOp::GE,
                                        right: Value::Identifier(var_b.clone()),
                                    },
                                    positive: vec![
                                        Command::Write {
                                            value: Value::Num(1),
                                        }
                                    ],
                                    negative: vec![
                                        Command::Write {
                                            value: Value::Num(0),
                                        }
                                    ],
                                },
                                Command::Assign {
                                    target: var_a.clone(),
                                    expr: Expression::Compound {
                                        left: Value::Identifier(var_a.clone()),
                                        op: ExprOp::Div,
                                        right: Value::Num(2),
                                    }
                                },
                            ],
                        }
                    ],
                }
            ],
        };

        assert_eq!(parsed.unwrap(), expected);
    }

    #[test]
    fn program1() {
        let text = r#"
            [ Eratostenes' sieve ]
            DECLARE
                n, j, sieve(2:100)
            BEGIN
                n ASSIGN 100;
                FOR i FROM n DOWNTO 2 DO
                    sieve(i) ASSIGN 1;
                ENDFOR
                FOR i FROM 2 TO n DO
                    IF sieve(i) NEQ 0 THEN
                        j ASSIGN i PLUS i;
                        WHILE j LEQ n DO
                            sieve(j) ASSIGN 0;
                            j ASSIGN j PLUS i;
                        ENDWHILE
                        WRITE i;
                    ENDIF
                ENDFOR
            END
        "#;

        let parsed = parse_ast(text);

        let var_n = Identifier::VarAccess { name: String::from("n") };
        let var_j = Identifier::VarAccess { name: String::from("j") };
        let temp_i = Identifier::VarAccess { name: String::from("i") };
        let var_sieve = String::from("sieve");

        let expected = ast::Program {
            declarations: Some(vec![
                Declaration::Var { name: String::from("n") },
                Declaration::Var { name: String::from("j") },
                Declaration::Array {
                    name: String::from("sieve"),
                    start: 2,
                    end: 100,
                },
            ]),
            commands: vec![
                Command::Assign {
                    target: var_n.clone(),
                    expr: Expression::Simple {
                        value: Value::Num(100),
                    },
                },
                Command::For {
                    counter: "i".to_string(),
                    ascending: false,
                    from: Value::Identifier(var_n.clone()),
                    to: Value::Num(2),
                    commands: vec![
                        Command::Assign {
                            target: Identifier::ArrAccess {
                                name: var_sieve.clone(),
                                index: String::from("i"),
                            },
                            expr: Expression::Simple {
                                value: Value::Num(1),
                            },
                        },
                    ],
                },
                Command::For {
                    counter: "i".to_string(),
                    ascending: true,
                    from: Value::Num(2),
                    to: Value::Identifier(var_n.clone()),
                    commands: vec![
                        Command::If {
                            condition: Condition {
                                left: Value::Identifier(Identifier::ArrAccess {
                                    name: var_sieve.clone(),
                                    index: String::from("i")
                                }),
                                op: RelOp::NEQ,
                                right: Value::Num(0),
                            },
                            positive: vec![
                                Command::Assign {
                                    target: var_j.clone(),
                                    expr: Expression::Compound {
                                        left: Value::Identifier(temp_i.clone()),
                                        op: ExprOp::Plus,
                                        right: Value::Identifier(temp_i.clone()),
                                    }
                                },
                                Command::While {
                                    condition: Condition {
                                        left: Value::Identifier(var_j.clone()),
                                        op: RelOp::LEQ,
                                        right: Value::Identifier(var_n.clone()),
                                    },
                                    commands: vec![
                                        Command::Assign {
                                            target: Identifier::ArrAccess {
                                                name: var_sieve.clone(),
                                                index: String::from("j"),
                                            },
                                            expr: Expression::Simple {
                                                value: Value::Num(0),
                                            },
                                        },
                                        Command::Assign {
                                            target: var_j.clone(),
                                            expr: Expression::Compound {
                                                left: Value::Identifier(var_j.clone()),
                                                op: ExprOp::Plus,
                                                right: Value::Identifier(temp_i.clone()),
                                            }
                                        },
                                    ],
                                },
                                Command::Write {
                                    value: Value::Identifier(temp_i.clone())
                                }
                            ],
                        }
                    ],
                },
            ],
        };

        assert_eq!(parsed.unwrap(), expected);
    }
}
