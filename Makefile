.PHONY: build test run

build:
	cargo build

run:
	cargo run --release

test:
	cargo test
