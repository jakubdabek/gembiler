use gembiler::code_generator::translator::Generator;
use gembiler::code_generator::intermediate;
use virtual_machine::interpreter;
use virtual_machine::interpreter::MemoryValue;

use std::fmt::{self, Write as _};

fn memval_vec<'a, I: IntoIterator<Item=&'a i64>>(iter: I) -> Vec<MemoryValue> {
    iter.into_iter().map(|v| interpreter::memval(*v)).collect()
}

fn big_memval_vec<'a, I: IntoIterator<Item=&'a str>, T>(iter: I) -> Vec<MemoryValue> {
    iter.into_iter().map(|v| v.parse().expect("invalid number")).collect()
}

fn print_readable_error<'a, I1, I2>(len: usize, output: I1, expected: I2) -> !
where I1: Iterator<Item=Option<&'a MemoryValue>>,
      I2: Iterator<Item=Option<&'a MemoryValue>> {
    let readable_message: Result<_, fmt::Error> = output
        .zip(expected)
        .fold(Ok(String::with_capacity(len * 23)), |buf, (out, exp)| {
            let mut buf = buf?;
            if let Some(out) = out {
                write!(buf, "{:-8}", out)?;
            } else {
                write!(buf, "  None  ")?;
            }

            write!(buf, " <=> ")?;

            if let Some(exp) = exp {
                write!(buf, "{}", exp)?;
            } else {
                write!(buf, "  None  ")?;
            }

            writeln!(buf, "")?;

            Ok(buf)
        });

    panic!("assertion failed: `(output == expected)`:\n{}", readable_message.expect("writing to String failed"));
}

fn check_success(code: &str, input: Vec<MemoryValue>, expected: &[MemoryValue]) {
    let ast = parser::parse_ast(code);
    assert!(ast.is_ok());
    let program = ast.unwrap();

    let ir = intermediate::generate(&program);
    assert!(ir.is_ok());

    let generator = Generator::new(ir.unwrap());
    let translated = generator.translate();
//    let (run_result, logs) = virtual_machine::interpreter::run_debug(translated, input, true);
    let run_result = virtual_machine::interpreter::run_extended(translated, input);

    println!("{:?}", run_result);
//    println!("{}", logs.join("\n"));

    let (_cost, output) = run_result.unwrap();

    if output != expected {
        let (output_len, expected_len) = (output.len(), expected.len());
        if output_len > expected_len {
            print_readable_error(output_len, &mut output.iter().map(Some), &mut expected.iter().map(Some).chain(std::iter::repeat(None)))
        } else {
            print_readable_error(expected_len, &mut output.iter().map(Some).chain(std::iter::repeat(None)), &mut expected.iter().map(Some))
        };


    }

    assert_eq!(output, expected);
}

#[test]
fn program0() {
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

    let input = memval_vec([10].iter());
    let expected = memval_vec([0i64, 1, 0, 1].iter());

    check_success(code, input, expected.as_slice());
}

#[test]
//#[ignore = "for not implemented"]
fn program1() {
    let code = r#"
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

    let input = memval_vec(&[]);
    let expected = memval_vec(&[2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97]);

    check_success(code, input, expected.as_slice());
}

#[test]
fn program2() {
    let code = r#"
        [ prime decomposition ]
        DECLARE
            n, m, remainder, exponent, divisor
        BEGIN
            READ n;
            divisor ASSIGN 2;
            m ASSIGN divisor TIMES divisor;
            WHILE n GEQ m DO
                exponent ASSIGN 0;
                remainder ASSIGN n MOD divisor;
                WHILE remainder EQ 0 DO
                    n ASSIGN n DIV divisor;
                    exponent ASSIGN exponent PLUS 1;
                    remainder ASSIGN n MOD divisor;
                ENDWHILE
                IF exponent GE 0 THEN [ divisor found? ]
                    WRITE divisor;
                    WRITE exponent;
                ELSE
                    divisor ASSIGN divisor PLUS 1;
                    m ASSIGN divisor TIMES divisor;
                ENDIF
            ENDWHILE
            IF n NEQ 1 THEN [ the last divisor ]
                WRITE n;
                WRITE 1;
            ENDIF
        END
    "#;

    let input = memval_vec(&[2]);
    let expected = memval_vec(&[2, 1]);

    check_success(code, input, expected.as_slice());

    let input = memval_vec(&[4]);
    let expected = memval_vec(&[2, 2]);

    check_success(code, input, expected.as_slice());

    let input = memval_vec(&[6]);
    let expected = memval_vec(&[2, 1, 3, 1]);

    check_success(code, input, expected.as_slice());

    let input = memval_vec(&[27]);
    let expected = memval_vec(&[3, 3]);

    check_success(code, input, expected.as_slice());

    let input = memval_vec(&[64]);
    let expected = memval_vec(&[2, 6]);

    check_success(code, input, expected.as_slice());
}

#[test]
fn div_mod() {
    let code = r#"
        [ div-mod.imp
          1 0
          1 0 0 0
        ]
        DECLARE
            a, b, c
        BEGIN
            READ a;
            READ b;
            c ASSIGN a DIV a;
            WRITE c;
            c ASSIGN a DIV b;
            WRITE c;
            c ASSIGN a MOD a;
            WRITE c;
            c ASSIGN a MOD b;
            WRITE c;
        END
    "#;

    let input = memval_vec(&[1, 0]);
    let expected = memval_vec(&[1, 0, 0, 0]);

    check_success(code, input, expected.as_slice());
}

#[test]
fn div_mod2() {
    let code = r#"
        [ div-mod.imp
          33 7
          4 5 -5 -2 4 -5 -5 2
        ]
        DECLARE
            a, b, c
        BEGIN
            READ a;
            READ b;
            c ASSIGN a DIV b;
            WRITE c;
            c ASSIGN a MOD b;
            WRITE c;
            b ASSIGN 0 MINUS b;
            c ASSIGN a DIV b;
            WRITE c;
            c ASSIGN a MOD b;
            WRITE c;
            a ASSIGN 0 MINUS a;
            c ASSIGN a DIV b;
            WRITE c;
            c ASSIGN a MOD b;
            WRITE c;
            b ASSIGN 0 MINUS b;
            c ASSIGN a DIV b;
            WRITE c;
            c ASSIGN a MOD b;
            WRITE c;
        END
    "#;

    let input = memval_vec(&[33, 7]);
    let expected = memval_vec(&[4, 5, -5, -2, 4, -5, -5, 2]);

    check_success(code, input, expected.as_slice());
}

#[test]
fn numbers() {
    let code = r#"
        [ numbers.imp - liczby ]
        DECLARE
            a, b, c, t(-6:6), d, e, f, g, h, i, j, tab(-5:5)
        BEGIN
            WRITE 0;
            WRITE 1;
            WRITE -2;
            WRITE 10;
            WRITE -100;
            WRITE 10000;
            WRITE -1234567890;

            a ASSIGN 1234566543;
            b ASSIGN -677777177;
            c ASSIGN 15;
            t(2) ASSIGN -555555555;
            d ASSIGN 8888;
            tab(-4) ASSIGN 11;
            t(0) ASSIGN -999;
            e ASSIGN 1111111111;
            tab(0) ASSIGN 7777;
            f ASSIGN -2048;
            g ASSIGN -123;
            t(-3) ASSIGN t(0);
            tab(-5) ASSIGN a;
            tab(-5) ASSIGN tab(0) DIV tab(-4);
            t(-5) ASSIGN tab(0);

            READ h;
            i ASSIGN 1;
            j ASSIGN h PLUS c;

            WRITE j; [ j = h + 15 ]
            WRITE c; [ c = 15 ]
            WRITE t(-3); [ -999 ]
            WRITE t(2); [ -555555555 ]
            WRITE t(-5); [ 7777 ]
            WRITE t(0); [ -999 ]
            WRITE tab(-4); [ 11 ]
            WRITE tab(-5); [ 707 ]
            WRITE tab(0); [ 7777 ]
        END
    "#;

    let h = 10;
    let input = memval_vec(&[h]);
    let expected = memval_vec(&[0, 1, -2, 10, -100, 10000, -1234567890, h + 15, 15, -999, -555555555, 7777, -999, 11, 707, 7777]);

    check_success(code, input, expected.as_slice());
}

#[test]
fn fib() {
    let code = r#"
        [ Fibonacci 26
        ? 1
        > 121393
        ]
        DECLARE
          tab(-987654321:1234567890),
          a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z
        BEGIN
          READ tab(-121212121);
          a ASSIGN tab(-121212121);
          b ASSIGN a;
          c ASSIGN b PLUS a;
          d ASSIGN c PLUS b;
          e ASSIGN d PLUS c;
          f ASSIGN e PLUS d;
          g ASSIGN f PLUS e;
          h ASSIGN g PLUS f;
          i ASSIGN h PLUS g;
          j ASSIGN i PLUS h;
          k ASSIGN j PLUS i;
          l ASSIGN k PLUS j;
          m ASSIGN l PLUS k;
          n ASSIGN m PLUS l;
          o ASSIGN n PLUS m;
          p ASSIGN o PLUS n;
          q ASSIGN p PLUS o;
          r ASSIGN q PLUS p;
          s ASSIGN r PLUS q;
          t ASSIGN s PLUS r;
          u ASSIGN t PLUS s;
          v ASSIGN u PLUS t;
          w ASSIGN v PLUS u;
          x ASSIGN w PLUS v;
          y ASSIGN x PLUS w;
          z ASSIGN y PLUS x;
          a ASSIGN 10000 TIMES z;
          tab(a) ASSIGN z;
          WRITE tab(a);
        END
    "#;

    let input = memval_vec(&[1]);
    let expected = memval_vec(&[121393]);

    check_success(code, input, expected.as_slice());
}

#[test]
fn fib_factorial() {
    let code = r#"
        [ Silnia  PLUS  Fibonacci
        ? 20
        > 2432902008176640000
        > 6765
        ]
        DECLARE
            f(0:100), s(0:100), i(0:100), n, k, l
        BEGIN
            READ n;
            f(0) ASSIGN 0;
            s(0) ASSIGN 1;
            i(0) ASSIGN 0;
            f(1) ASSIGN 1;
            s(1) ASSIGN 1;
            i(1) ASSIGN 1;
            FOR j FROM 2 TO n DO
            k ASSIGN j MINUS 1;
                l ASSIGN k MINUS 1;
            i(j) ASSIGN i(k) PLUS 1;
            f(j) ASSIGN f(k) PLUS f(l);
                s(j) ASSIGN s(k) TIMES i(j);
            ENDFOR
            WRITE s(n);
            WRITE f(n);
        END
    "#;

    let input = memval_vec(&[20]);
    let expected = memval_vec(&[2432902008176640000, 6765]);

    check_success(code, input, expected.as_slice());
}

#[test]
fn factorial() {
    let code = r#"
        [ Silnia
        ? 20
        > 2432902008176640000
        ]
        DECLARE
          s(0:100), n, m, a, j
        BEGIN
            READ n;
            s(0) ASSIGN 1;
            m ASSIGN n;
            FOR i FROM 1 TO m DO
                a ASSIGN i MOD 2;
                j ASSIGN i MINUS 1;
                IF a EQ 1 THEN
                    s(i) ASSIGN s(j) TIMES m;
                ELSE
                    s(i) ASSIGN m TIMES s(j);
                ENDIF
                m ASSIGN m MINUS 1;
            ENDFOR
            WRITE s(n);
        END
    "#;

    let input = memval_vec(&[20]);
    let expected = memval_vec(&[2432902008176640000]);

    check_success(code, input, expected.as_slice());
}

#[test]
fn tab() {
    let code = r#"
        [ tab.imp ]
        DECLARE
            n, j, ta(0:25), tb(0:25), tc(0:25)
        BEGIN
            n  ASSIGN  25;
            tc(0)  ASSIGN  n;
            tc(n)  ASSIGN  n MINUS n;
            FOR i FROM tc(0) DOWNTO tc(n) DO
                ta(i)  ASSIGN  i;
                tb(i)  ASSIGN  n MINUS i;
            ENDFOR
            FOR i FROM tc(n) TO tc(0) DO
                tc(i)  ASSIGN  ta(i) TIMES tb(i);
            ENDFOR
            FOR i FROM 0 TO n DO
                WRITE tc(i);
            ENDFOR
        END
    "#;

    let input = memval_vec(&[]);

    let n = 25;
    let tc: Vec<_> = (0..=n).map(|i| i * (n - i)).collect();
    let expected = memval_vec(&tc);

    check_success(code, input, expected.as_slice());
}

#[test]
fn mod_mult() {
    let code = r#"
        [ a ^ b mod c
        ? 1234567890
        ? 1234567890987654321
        ? 987654321
        > 674106858
        ]
        DECLARE
            a, b, c, wynik, pot, wybor
        BEGIN
            READ a;
            READ b;
            READ c;
            wynik ASSIGN 1;
            pot ASSIGN a MOD c;
            WHILE b GE 0 DO
                wybor ASSIGN b MOD 2;
                IF wybor EQ 1 THEN
                    wynik ASSIGN wynik TIMES pot;
                    wynik ASSIGN wynik MOD c;
                ENDIF
                b ASSIGN b DIV 2;
                pot ASSIGN pot TIMES pot;
                pot ASSIGN pot MOD c;
            ENDWHILE
            WRITE wynik;
        END
    "#;

    let input = memval_vec(&[1234567890, 1234567890987654321, 987654321]);
    let expected = memval_vec(&[674106858]);

    check_success(code, input, expected.as_slice());
}

#[test]
fn loopiii() {
    let code = r#"
        [ loopiii.imp - nested loops
            0 0 0
            31000 40900 2222010

            1 0 2
            31001 40900 2222012
        ]
        DECLARE
            a, b, c
        BEGIN
            READ a;
            READ b;
            READ c;
            FOR i FROM 111091 TO 111110 DO
                FOR j FROM 209 DOWNTO 200 DO
                    FOR k FROM 11 TO 20 DO
                        a  ASSIGN  a PLUS k;
                    ENDFOR
                    b  ASSIGN  b PLUS j;
                ENDFOR
                c  ASSIGN  c PLUS i;
            ENDFOR
            WRITE a;
            WRITE b;
            WRITE c;
        END
    "#;

    let input = memval_vec(&[0, 0, 0]);
    let expected = memval_vec(&[31000, 40900, 2222010]);

    check_success(code, input, expected.as_slice());

    let input = memval_vec(&[1, 0, 2]);
    let expected = memval_vec(&[31001, 40900, 2222012]);

    check_success(code, input, expected.as_slice());
}

#[test]
fn for_loop() {
    let code = r#"
        [ for.imp
          12 23 34
          507 4379 0
        ]
        DECLARE
            a, b, c
        BEGIN
            READ a;
            READ b;
            READ c;
            FOR i FROM 9 DOWNTO 0 DO
                FOR j FROM 0 TO i DO
                    FOR k FROM 0 TO j DO
                        a  ASSIGN  a PLUS k;
                        c  ASSIGN  k TIMES j;
                        c  ASSIGN  c PLUS i;
                        b  ASSIGN  b PLUS c;
                    ENDFOR
                ENDFOR
            ENDFOR
            WRITE a;
            WRITE b;
            WRITE c;
        END
    "#;

    let input = memval_vec(&[12, 23, 34]);
    let expected = memval_vec(&[507, 4379, 0]);

    check_success(code, input, expected.as_slice());
}

#[test]
#[ignore = "unknown result"]
fn sort() {
    let code = r#"
        [ sort.imp ]
        DECLARE
            tab(-11:10), x, q, w, j, k, n, a, b
        BEGIN
            a ASSIGN -11;
            b ASSIGN 10;
            n ASSIGN 23;
            q ASSIGN 5;
            w ASSIGN 1;
            [generate unsorted array]
            FOR i FROM b DOWNTO a DO
                w  ASSIGN  w TIMES q;
                w  ASSIGN  w MOD n;
                tab(i)  ASSIGN  w;
            ENDFOR
            [display unsorted array]
            FOR i FROM a TO b DO
                WRITE tab(i);
            ENDFOR
            WRITE 1234567890;
            [sort]
                q ASSIGN a PLUS 1;
            FOR i FROM q TO b DO
                x  ASSIGN  tab(i);
                j  ASSIGN  i;
                WHILE j GE a DO
                    k  ASSIGN  j MINUS 1;
                    IF tab(k) GE x THEN
                        tab(j)  ASSIGN  tab(k);
                        j  ASSIGN  j MINUS 1;
                    ELSE
                        k  ASSIGN  j;
                        j  ASSIGN  a;
                    ENDIF
                ENDWHILE
                tab(k)  ASSIGN  x;
            ENDFOR
            [display sorted array]
            FOR i FROM a TO b DO
                WRITE tab(i);
            ENDFOR
        END
    "#;

    let input = memval_vec(&[]);
    let expected = memval_vec(&[]); //?

    check_success(code, input, expected.as_slice());
}
