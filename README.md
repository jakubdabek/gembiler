# Gembiler

This is a compiler for a simple language (TBA: definition of said language)

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
