use num_integer::Integer;

pub struct Data {
    text: &'static str,
    exec_fn: fn(Vec<i64>) -> Vec<i64>,
}

impl Data {
    pub fn text(&self) -> &'static str {
        self.text
    }

    pub fn exec(&self, mut input: Vec<i64>) -> Vec<i64> {
        input.reverse();
        (self.exec_fn)(input)
    }
}

mod offset_collection;
use crate::data::offset_collection::{UninitializedCollection, OffsetCollection};

fn do_div<T: Integer>(a: T, b: T) -> T {
    if b.is_zero() {
        T::zero()
    } else {
        a.div_floor(&b)
    }
}

fn do_mod<T: Integer>(a: T, b: T) -> T {
    if b.is_zero() {
        T::zero()
    } else {
        a.mod_floor(&b)
    }
}

fn new_collection(bottom: i64, top: i64) -> OffsetCollection<UninitializedCollection<Vec<Option<i64>>>> {
    let init = std::iter::repeat(None).take((top - bottom + 1) as usize);
    OffsetCollection::new(UninitializedCollection::new(init.collect()), -bottom)
}

const BITSTRING_TEXT: &str = r#"
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

pub const BITSTRING_DATA: Data = Data {
    text: BITSTRING_TEXT,
    exec_fn: |mut input| {
        let mut output = vec![];

        let a = input.pop().expect("invalid input");

        if a >= 0 {
            let mut a = a as u64;
            while a > 0 {
                let b = a & !1;
                if a > b {
                    output.push(1);
                } else {
                    output.push(0);
                }
                a = do_div(a, 2);
            }
        }

        assert!(input.is_empty());
        output
    },
};

const SIEVE_TEXT: &str = r#"
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

pub const SIEVE_DATA: Data = Data {
    text: SIEVE_TEXT,
    exec_fn: |input| {
        let mut output = vec![];

        let mut sieve = new_collection(2, 100);

        let n = 100;
        for i in (2..=n).rev() {
            sieve[i] = 1;
        }
        for i in 2..=n {
            if sieve[i] != 0 {
                let mut j = 2 * i;
                while j <= n {
                    sieve[j] = 0;
                    j += i;
                }
                output.push(i);
            }
        }

        assert!(input.is_empty());
        output
    },
};


const PRIME_DECOMPOSITION_TEXT: &str = r#"
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

pub const PRIME_DECOMPOSITION_DATA: Data = Data {
    text: PRIME_DECOMPOSITION_TEXT,
    exec_fn: |mut input| {
        let mut n = input.pop().expect("invalid input");

        let mut divisor = 2;
        let mut m = divisor * divisor;

        let mut output = vec![];

        while n >= m {
            let mut exponent = 0;
            let mut remainder = n.mod_floor(&divisor);

            while remainder == 0 {
                n = n.div_floor(&divisor);
                exponent += 1;
                remainder = n.mod_floor(&divisor);
            }

            if exponent > 0 {
                output.push(divisor);
                output.push(exponent);
            } else {
                divisor += 1;
                m = divisor * divisor;
            }
        }

        if n != 1 { // the last divisor
            output.push(n);
            output.push(1);
        }

        assert!(input.is_empty());
        output
    },
};

const DIV_MOD_TEXT: &str = r#"
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

pub const DIV_MOD_DATA: Data = Data {
    text: DIV_MOD_TEXT,
    exec_fn: |mut input| {
        let a = input.pop().expect("invalid input");
        let b = input.pop().expect("invalid input");

        assert!(input.is_empty());
        vec![
            do_div(a, a),
            do_div(a, b),
            do_mod(a, a),
            do_mod(a, b),
        ]
    }
};

const DIV_MOD2_TEXT: &str = r#"
    DECLARE
        a, b, c, choice
    BEGIN
        READ choice;
        IF choice > 0 THEN
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
        ENDIF
    END
"#;

pub const DIV_MOD2_DATA: Data = Data {
    text: DIV_MOD2_TEXT,
    exec_fn: |mut input| {
        let mut output = vec![];
        let mut choice = input.pop().expect("invalid input");

        while choice > 0 {
            let a = input.pop().expect("invalid input");
            let b = input.pop().expect("invalid input");

            output.extend_from_slice(&[
                do_div( a,  b),
                do_mod( a,  b),
                do_div( a, -b),
                do_mod( a, -b),
                do_div(-a, -b),
                do_mod(-a, -b),
                do_div(-a,  b),
                do_mod(-a,  b),
            ]);

            choice = input.pop().expect("invalid input");
        }

        assert!(input.is_empty());
        output
    }
};

const NUMBERS_TEXT: &str = r#"
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

pub const NUMBERS_DATA: Data = Data {
    text: NUMBERS_TEXT,
    exec_fn: |mut input| {
        let mut output = vec![
            0,
            1,
            -2,
            10,
            -100,
            10000,
            -1234567890,
        ];

        let mut t = new_collection(-6, 6);
        let mut tab = new_collection(-5, 5);

        let a = 1234566543;
        let b = -677777177;
        let c = 15;
        t[2] = -555555555;
        let d = 8888;
        tab[-4] = 11;
        t[0] = -999;
        let e = 1111111111;
        tab[0] = 7777;
        let f = -2048;
        let g = -123;
        t[-3] = t[0];
        tab[-5] = a;
        tab[-5] = do_div(tab[0], tab[-4]);
        t[-5] = tab[0];

        let h = input.pop().expect("invalid input");
        let i = 1;
        let j = h + c;

        output.push(j);
        output.push(c);
        output.push(t[-3]);
        output.push(t[2]);
        output.push(t[-5]);
        output.push(t[0]);
        output.push(tab[-4]);
        output.push(tab[-5]);
        output.push(tab[0]);

        output
    }
};

const FIB_TEXT: &str = r#"
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

pub const FIB_DATA: Data = Data {
    text: FIB_TEXT,
    exec_fn: |mut input| {
        let a = input.pop().expect("invalid input");
        let b = a;
        let c = b + a;
        let d = c + b;
        let e = d + c;
        let f = e + d;
        let g = f + e;
        let h = g + f;
        let i = h + g;
        let j = i + h;
        let k = j + i;
        let l = k + j;
        let m = l + k;
        let n = m + l;
        let o = n + m;
        let p = o + n;
        let q = p + o;
        let r = q + p;
        let s = r + q;
        let t = s + r;
        let u = t + s;
        let v = u + t;
        let w = v + u;
        let x = w + v;
        let y = x + w;
        let z = y + x;

        assert!(input.is_empty());
        vec![z]
    },
};

const FIB_FACTORIAL_TEXT: &str = r#"
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

pub const FIB_FACTORIAL_DATA: Data = Data {
    text: FIB_FACTORIAL_TEXT,
    exec_fn: |mut input| {
        let mut f = new_collection(0, 100);
        let mut s = new_collection(0, 100);
        let mut i = new_collection(0, 100);

        let n = input.pop().expect("invalid input");
        f[0] = 0;
        s[0] = 1;
        i[0] = 0;
        f[1] = 1;
        s[1] = 1;
        i[1] = 1;

        for j in 2..=n {
            let k = j - 1;
            let l = k - 1;
            i[j] = i[k] + 1;
            f[j] = f[k] + f[l];
            s[j] = s[k] * i[j];
        }

        assert!(input.is_empty());
        vec![s[n], f[n]]
    },
};

const FACTORIAL_TEXT: &str = r#"
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

pub const FACTORIAL_DATA: Data = Data {
    text: FACTORIAL_TEXT,
    exec_fn: |mut input| {
        let n = input.pop().expect("invalid input");
        let mut s = new_collection(0, 100);

        s[0] = 1;
        let mut m = n;

        for i in 1..=m {
            let a = do_mod(i, 2);
            let j = i - 1;

            if a == 1 {
                s[i] = s[j] * m;
            } else {
                s[i] = m * s[j];
            }

            m -= 1;
        }

        assert!(input.is_empty());
        vec![s[n]]
    },
};

const TAB_TEXT: &str = r#"
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

pub const TAB_DATA: Data = Data {
    text: TAB_TEXT,
    exec_fn: |input| {
        (0..=25).map(|v| v * (25 - v)).collect()
    },
};

const MOD_MULT_TEXT: &str = r#"
    [ a ^ b mod c
    ? 1234567890
    ? 1234567890987654321
    ? 987654321
    > 674106858
    ]
    DECLARE
        a, b, c, result, exponent, choice
    BEGIN
        READ a;
        READ b;
        READ c;
        result ASSIGN 1;
        exponent ASSIGN a MOD c;
        WHILE b GE 0 DO
            choice ASSIGN b MOD 2;
            IF choice EQ 1 THEN
                result ASSIGN result TIMES exponent;
                result ASSIGN result MOD c;
            ENDIF
            b ASSIGN b DIV 2;
            exponent ASSIGN exponent TIMES exponent;
            exponent ASSIGN exponent MOD c;
        ENDWHILE
        WRITE result;
    END
"#;

pub const MOD_MULT_DATA: Data = Data {
    text: MOD_MULT_TEXT,
    exec_fn: |mut input| {
        let a = input.pop().expect("invalid input");
        let mut b = input.pop().expect("invalid input");
        let c = input.pop().expect("invalid input");

        let mut result = 1;
        let mut exponent = do_mod(a, c);

        while b > 0 {
            let choice = do_mod(b, 2);

            if choice == 1 {
                result = do_mod(result * exponent, c);
            }

            b = do_div(b, 2);
            exponent = do_mod(exponent * exponent, c);
        }

        assert!(input.is_empty());
        vec![result]
    },
};

const LOOPIII_TEXT: &str = r#"
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

pub const LOOPIII_DATA: Data = Data {
    text: LOOPIII_TEXT,
    exec_fn: |mut input| {
        let mut a = input.pop().expect("invalid input");
        let mut b = input.pop().expect("invalid input");
        let mut c = input.pop().expect("invalid input");

        for i in 111091..=111110 {
            for j in (200..=209).rev() {
                for k in 11..=20 {
                    a = a + k;
                }
                b = b + j;
            }
            c = c + i;
        }

        assert!(input.is_empty());
        vec![a, b, c]
    },
};

const FOR_TEXT: &str = r#"
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

pub const FOR_DATA: Data = Data {
    text: FOR_TEXT,
    exec_fn: |mut input| {
        let mut a = input.pop().expect("invalid input");
        let mut b = input.pop().expect("invalid input");
        let mut c = input.pop().expect("invalid input");

        for i in (0..=9).rev() {
            for j in 0..=i {
                for k in 0..=j {
                    a = a + k;
                    c = k * j;
                    c = c + i;
                    b = b + c;
                }
            }
        }

        assert!(input.is_empty());
        vec![a, b, c]
    },
};

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bitstring() {
        fn rev(s: Vec<i64>) -> Vec<i64> {
            s.into_iter().rev().collect()
        }
        assert_eq!(BITSTRING_DATA.exec(vec![0b1010]), rev(vec![1, 0, 1, 0]));
        assert_eq!(BITSTRING_DATA.exec(vec![0b1110]), rev(vec![1, 1, 1, 0]));
        assert_eq!(BITSTRING_DATA.exec(vec![0b1111]), rev(vec![1, 1, 1, 1]));
        assert_eq!(BITSTRING_DATA.exec(vec![0b10000]), rev(vec![1, 0, 0, 0, 0]));
        assert_eq!(BITSTRING_DATA.exec(vec![0b11001]), rev(vec![1, 1, 0, 0, 1]));
        assert_eq!(BITSTRING_DATA.exec(vec![1345601]), rev(vec![1, 0, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1]));
    }

    #[test]
    fn sieve() {
        assert_eq!(SIEVE_DATA.exec(vec![]), &[2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97]);
    }

    #[test]
    fn prime_decomposition() {
        assert_eq!(PRIME_DECOMPOSITION_DATA.exec(vec![2]), &[2, 1]);
        assert_eq!(PRIME_DECOMPOSITION_DATA.exec(vec![3]), &[3, 1]);
        assert_eq!(PRIME_DECOMPOSITION_DATA.exec(vec![4]), &[2, 2]);
        assert_eq!(PRIME_DECOMPOSITION_DATA.exec(vec![9]), &[3, 2]);
        assert_eq!(PRIME_DECOMPOSITION_DATA.exec(vec![15]), &[3, 1, 5, 1]);
        assert_eq!(PRIME_DECOMPOSITION_DATA.exec(vec![16]), &[2, 4]);
        assert_eq!(PRIME_DECOMPOSITION_DATA.exec(vec![17]), &[17, 1]);
        assert_eq!(PRIME_DECOMPOSITION_DATA.exec(vec![20]), &[2, 2, 5, 1]);
        assert_eq!(PRIME_DECOMPOSITION_DATA.exec(vec![22]), &[2, 1, 11, 1]);
        assert_eq!(PRIME_DECOMPOSITION_DATA.exec(vec![27]), &[3, 3]);
        assert_eq!(PRIME_DECOMPOSITION_DATA.exec(vec![64]), &[2, 6]);
        assert_eq!(PRIME_DECOMPOSITION_DATA.exec(vec![12345654321]), &[3, 2, 7, 2, 11, 2, 13, 2, 37, 2]);
    }

    #[test]
    fn div_mod() {
        assert_eq!(DIV_MOD_DATA.exec(vec![1, 0]), &[1, 0, 0, 0]);
        assert_eq!(DIV_MOD_DATA.exec(vec![1, 2]), &[1, 0, 0, 1]);
    }

    #[test]
    fn div_mod2() {
        assert_eq!(DIV_MOD2_DATA.exec(vec![1, 33, 7, 0]), &[4, 5, -5, -2, 4, -5, -5, 2]);
        assert_eq!(
            DIV_MOD2_DATA.exec(vec![
                1, 33, 7,
                1, 33, 8,
                1, 33, 9,
                1, 33, 10,
                1, 12, 14,
                0,
            ]),
            vec![
                4, 5,
                -5, -2,
                4, -5,
                -5, 2,

                4, 1,
                -5, -7,
                4, -1,
                -5, 7,

                3, 6,
                -4, -3,
                3, -6,
                -4, 3,

                3, 3,
                -4, -7,
                3, -3,
                -4, 7,

                0, 12,
                -1, -2,
                0, -12,
                -1, 2,
            ]
        );
    }

    #[test]
    fn numbers() {
        for h in -20..=20 {
            assert_eq!(NUMBERS_DATA.exec(vec![h]), vec![0, 1, -2, 10, -100, 10000, -1234567890, h + 15, 15, -999, -555555555, 7777, -999, 11, 707, 7777])
        }
    }

    #[test]
    fn fib() {
        assert_eq!(FIB_DATA.exec(vec![1]), &[121393]);
    }

    #[test]
    fn fib_factorial() {
        assert_eq!(FIB_FACTORIAL_DATA.exec(vec![20]), &[2432902008176640000, 6765]);
    }

    #[test]
    fn factorial() {
        assert_eq!(FACTORIAL_DATA.exec(vec![20]), &[2432902008176640000]);
    }

    #[test]
    fn mod_mult() {
        assert_eq!(MOD_MULT_DATA.exec(vec![1234567890, 1234567890987654321, 987654321]), &[674106858]);
    }

    #[test]
    fn loopiii() {
        assert_eq!(LOOPIII_DATA.exec(vec![0, 0, 0]), &[31000, 40900, 2222010]);
        assert_eq!(LOOPIII_DATA.exec(vec![1, 0, 2]), &[31001, 40900, 2222012]);
    }

    #[test]
    fn for_test() {
        assert_eq!(FOR_DATA.exec(vec![12, 23, 34]), &[507, 4379, 0]);
    }
}
