use gembiler::code_generator::translator::translate;
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

    let translated = translate(ir.unwrap());
    let run_result = virtual_machine::interpreter::run(translated, vec![interpreter::memval(9)]);
    println!("{:?}", run_result);
    let (output, _cost) = run_result.unwrap();
    let expected: Vec<_> = [0i64, 0, 1, 1].iter().map(|&v| interpreter::memval(v)).collect();
    assert_eq!(output, expected);
}
