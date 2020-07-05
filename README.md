# Gembiler

This is a compiler for a simple language made for university course "Formal Languages & Translation Techniques".

## Building

1. Install Rust from the [rustup page](https://rustup.rs/)
    ```shell script
    curl -sSf https://sh.rustup.rs | sh
    ```

2. Run Makefile
    ```shell script
    make
    ```

3. You can now run the executable with `./gembiler` and interpret its output with `./interpreter`

Instead of steps _2._ and _3._, you can use `cargo` for building and running the code:

```shell script
cargo build --workspace [--release]
cargo run --package gembiler <in file> <out file>
cargo run --package virtual-machine <gembiler output>
```

## Modules

The compiler infrastructure is split into modules:

1.  `parser` is responsible for lexing the source files and parsing them
    into an AST derived from the grammar.
1.  The main crate `gembiler` takes the AST and produces VM instructions.  
    `verifier` makes a pass over the AST and detects semantic errors.  
    `code_generator` uses an intermediate representation (also an AST) to
    more easily optimize the code and then translate it into instructions for the VM.
1.  `test-data` contains code for testing example programs.
1.  `virtual-machine` interprets the result code, either programatically using
    the API or by reading from a file.

## The language

This is the language's grammar in EBNF:

```ebnf
program       = "DECLARE" , declarations , "BEGIN" , commands , "END"
              | "BEGIN" , commands , "END";

declarations  = declarations , "," , pidentifier
              | declarations , "," , pidentifier , "(" , num , ":" , num , ")"
              | pidentifier
              | pidentifier , "(" , num , ":" , num , ")";

commands      = commands , "," , command
              | command;

command       = identifier , "ASSIGN" , expression , ";"
              | "IF" , condition , "THEN" , commands , "ELSE" , commands , "ENDIF"
              | "IF" , condition , "THEN" , commands , "ENDIF"
              | "WHILE" , condition , "DO" , commands , "ENDWHILE"
              | "DO" , commands , "WHILE" , condition , "ENDDO"
              | "FOR" , pidentifier , "FROM" , value , "TO" , value , "DO" , commands , "ENDFOR"
              | "FOR" , pidentifier , "FROM" , value , "DOWNTO" , value , "DO" , commands , "ENDFOR"
              | "READ" , identifier , ";"
              | "WRITE" , value , ";";

expression    = value
              | value , "PLUS" , value
              | value , "MINUS" , value
              | value , "TIMES" , value
              | value , "DIV" , value
              | value , "MOD" , value;

condition     = value , "EQ" , value
              | value , "NEQ" , value
              | value , "LE" , value
              | value , "GE" , value
              | value , "LEQ" , value
              | value , "GEQ" , value;

value         = num
              | identifier;

identifier    = pidentifier
              | pidentifier , "(" , pidentifier , ")"
              | pidentifier , "(" , num , ")";

pidentifier   = r"[_a-z]+";
num           = r"-?[0-9]+";
```

Numbers described by `num` are limited to signed 64-bit integers.  
There are comments allowed, delimited by square brackets, e.g. `[comment]`;
they cannot be nested.  
All whitespace is ignored.

### Rules for commands and expressions

1.  `PLUS`, `MINUS`, `TIMES`, `DIV`, `MOD` mean respectively
    integer addition, subtraction, multiplication, floor division and modulo.
    The result of division and modulo with divisor `0` should also give result `0`.
1.  `EQ`, `NEQ`, `LE`, `LEQ`, `GE`, `GEQ` mean respectively
    `==`, `!=`, `<`, `<=`, `>`, `>=`.
1.  `ident ASSIGN expr` assigns the value of `expr` to `ident`.
1.  A declaration `tab(-10:100)` describes an array of 111 elements,
    with `tab(-10)` being the first one, and `tab(100)` being the last one.
    It is an error to declare an array `tab(a:b)` with `a > b`.
1.  The `FOR` command's iterator variable is local and cannot be modified
    inside the loop with an `ASSIGN` command.  
    The loop's bounds are calculated before entering the loop and mutating
    the variables containing these bounds doesn't change the number of iterations.  
    The iterator variable is increased or decreased by 1 on each iteration
    depending on usage of `TO` and `DOWNTO`.
1.  `READ` and `WRITE` operate on the world by reading and writing numbers usually
    from and to the console.


## Virtual machine

The virtual machine has and instruction pointer `IP` and a memory `M` with
consecutive cells `M(0)`, `M(1)`, ... containing either 64-bit integers or
arbitrary precision integers with the `bignum` feature.
There are no registers, only memory cells, with `M(0)` treated as the accumulator.

### Instructions

Comments start with `#` and continue to the end of the line.
Other whitespace is ignored.  
Each instruction increments `IP` by one unless otherwise stated.
The first instruction starts at `IP = 0`.  
The whole memory (including `M(0)`) is treated as uninitialized at the start of a program.
Executing a nonexistent instruction or reading uninitialized memory results in an error.


| Instruction | Meaning | Cost |
|-------------|---------|-----:|
| `GET` | `M(0) <- a number from stdin` | 100 |
| `PUT` | `stdout <- M(0)`              | 100 |
| |
| `LOAD i`      | `M(0) <- M(i)`    | 10 |
| `STORE i`     | `M(i) <- M(0)`    | 10 |
| `LOADI i`     | `M(0) <- M(M(i))` | 20 |
| `STOREI i`    | `M(M(i)) <- M(0)` | 20 |
| |
| `ADD i`   | `M(0) <- M(0) + M(i)`             | 10 |
| `SUB i`   | `M(0) <- M(0) - M(i)`             | 10 |
| `SHIFT i` | `M(0) <- floor(M(0) * 2^M(i))`    |  5 |
| `INC`     | `M(0) <- M(0) + 1`                |  1 |
| `DEC`     | `M(0) <- M(0) - 1`                |  1 |
| |
| `JUMP j`  | `IP <- j`                                     | 1 |
| `JPOS j`  | `if M(0) > 0 then IP <- j else IP <- IP + 1`  | 1 |
| `JZERO j` | `if M(0) = 0 then IP <- j else IP <- IP + 1`  | 1 |
| `JNEG j`  | `if M(0) < 0 then IP <- j else IP <- IP + 1`  | 1 |
| |
| `HALT` | stop execution | 0 |
