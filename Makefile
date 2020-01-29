all: build

build:
	cargo build --release --workspace
	cp target/release/gembiler target/release/interpreter ./

.PHONY: all build
