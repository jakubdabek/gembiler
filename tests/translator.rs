use gembiler::code_generator::translator::Generator;
use gembiler::code_generator::intermediate;
use virtual_machine::interpreter;

#[test]
fn it_works() {
    let code = r#"
        DECLARE
            a, b, tab(5:100), tabsmall(10:20)
        BEGIN
            READ a;
            IF a GEQ 0 THEN
                WHILE a GE 0 DO
                    b ASSIGN a DIV 2;
                    b ASSIGN 2 TIMES b;
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

    let ast = parser::parse_ast(code);
    assert!(ast.is_ok());
    let program = ast.unwrap();

    let ir = intermediate::generate(&program);
    assert!(ir.is_ok());

    let generator = Generator::new(ir.unwrap());
    let translated = generator.translate();
    let (run_result, logs) = virtual_machine::interpreter::run_debug(translated, vec![interpreter::memval(10)], true);

    println!("{:?}", run_result);
    println!("{}", logs.join("\n"));

    let (_cost, output) = run_result.unwrap();
    let expected: Vec<_> = [0i64, 1, 0, 1].iter().map(|&v| interpreter::memval(v)).collect();

    assert_eq!(output, expected);
}
