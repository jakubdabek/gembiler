use gembiler::code_generator::translator::translate;
use gembiler::code_generator::intermediate;

#[test]
fn it_works() {
    let code = r#"
        DECLARE
            a, b
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
    let run_result = virtual_machine::interpreter::run(translated, vec![9]);
    println!("{:?}", run_result);
    let (output, _cost) = run_result.unwrap();
    assert_eq!(output, &[0, 0, 1, 1]);
}
