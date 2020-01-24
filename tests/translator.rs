use gembiler::code_generator::translator::Generator;
use gembiler::code_generator::intermediate;
use virtual_machine::interpreter;
use virtual_machine::interpreter::{MemoryValue, memval};
use test_data::TEST_DATA;

use std::fmt::{self, Write as _, Display, Formatter, Error, Debug};

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

struct DebugMultilineCollectionPrinter<'a, I>(&'a I) where &'a I: IntoIterator;
struct DebugDisplayWrapper<T>(T);

impl <T: Display> Debug for DebugDisplayWrapper<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

impl <'a, I> Debug for DebugMultilineCollectionPrinter<'a, I>
    where
        &'a I: IntoIterator,
        <&'a I as IntoIterator>::Item: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.debug_list().entries(self.0.into_iter().map(DebugDisplayWrapper)).finish()
    }
}

fn check_success(code: &str, input: Vec<MemoryValue>, expected: &[MemoryValue]) {
    let ast = parser::parse_ast(code);
    assert!(ast.is_ok());
    let program = ast.unwrap();

    let ir = intermediate::generate(&program);
    assert!(ir.is_ok());

    println!("{:#?}", DebugMultilineCollectionPrinter(&input));

    let generator = Generator::new(ir.unwrap());
    let translated = generator.translate();
    let (run_result, logs) = virtual_machine::interpreter::run_debug(translated, input, true);
//    let run_result = virtual_machine::interpreter::run_extended(translated, input);

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

macro_rules! make_test {
    ($test_name:ident) => {
        #[test]
        fn $test_name() {
            let data = &TEST_DATA[stringify!($test_name)];

            for (input, expected) in data.valid_io.iter() {
                check_success(
                    data.text,
                    memval_vec(input),
                    memval_vec(expected).as_slice()
                );
            }
        }
    }
}

make_test!(bitstring);
make_test!(sieve);
make_test!(prime_decomposition);
make_test!(div_mod);
make_test!(div_mod2);
make_test!(numbers);
make_test!(fib);
make_test!(factorial);
make_test!(fib_factorial);
make_test!(tab);
make_test!(mod_mult);
make_test!(loopiii);
make_test!(for_loop);

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

#[test]
#[ignore = "unknown result"]
fn bonus0() {
    let code = r#"
        DECLARE
            a, b, c, d, e, f, g, h, x
        BEGIN
            READ a;
            READ b;
            READ c;
            READ d;
            READ e;
            READ f;
            READ g;
            READ h;
            x ASSIGN 0;
            FOR i FROM 1 TO a DO
                FOR j FROM -1 DOWNTO b DO
                    FOR k FROM 1 TO c DO
                        FOR l FROM -1 DOWNTO d DO
                            FOR m FROM 1 TO e DO
                                FOR n FROM -1 DOWNTO f DO
                                    FOR o FROM 1 TO g DO
                                        FOR p FROM -1 DOWNTO h DO
                                            x ASSIGN x PLUS 1;
                                        ENDFOR
                                    ENDFOR
                                ENDFOR
                            ENDFOR
                        ENDFOR
                    ENDFOR
                ENDFOR
            ENDFOR
        END
    "#;

    let input = memval_vec(&[]);
    let expected = memval_vec(&[]); //?

    check_success(code, input, expected.as_slice());
}

#[test]
#[ignore = "too long"]
fn bonus2() {
    let code = r#"
        [ Rozklad liczby 340282367713220089251654026161790386200 na czynniki pierwsze ]
        [ Oczekiwany wynik:
          2^3
          3
          5^2
          7
          13
          41
          61
          641
          1321
          6700417
          613566757
          715827883
        ]
        DECLARE
            a(0:3),
            n, m, reszta, potega, dzielnik
        BEGIN
            a(0) ASSIGN 4294967297;
            a(1) ASSIGN 4294967298;
            a(2) ASSIGN 4294967299;
            a(3) ASSIGN 4294967300;

            n ASSIGN a(0) TIMES a(1);
            n ASSIGN n TIMES a(2);
            n ASSIGN n TIMES a(3);

            dzielnik ASSIGN 2;
            m ASSIGN dzielnik TIMES dzielnik;
            WHILE n GEQ m DO
                potega ASSIGN 0;
                reszta ASSIGN n MOD dzielnik;
                WHILE reszta EQ 0 DO
                    n ASSIGN n DIV dzielnik;
                    potega ASSIGN potega PLUS 1;
                    reszta ASSIGN n MOD dzielnik;
                ENDWHILE
                IF potega GE 0 THEN [ czy znaleziono dzielnik ]
                    WRITE dzielnik;
                    WRITE potega;
                ELSE
                    dzielnik ASSIGN dzielnik PLUS 1;
                    m ASSIGN dzielnik TIMES dzielnik;
                ENDIF
            ENDWHILE
            IF n NEQ 1 THEN [ ostatni dzielnik ]
                WRITE n;
                WRITE 1;
            ENDIF
        END
    "#;

    let input = memval_vec(&[]);
    let expected = memval_vec(&[
        2, 3,
        3, 1,
        5, 2,
        7, 1,
        13, 1,
        41, 1,
        61, 1,
        641, 1,
        1321, 1,
        6700417, 1,
        613566757, 1,
        715827883, 1,
    ]);

    check_success(code, input, expected.as_slice());
}
