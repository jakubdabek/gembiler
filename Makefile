all: build

build:
	cargo build --release --package gembiler
	cp target/release/gembiler ./compiler

.PHONY: all build
