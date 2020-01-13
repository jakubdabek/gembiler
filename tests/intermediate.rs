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
    println!("{:?}", ir.unwrap());
}
