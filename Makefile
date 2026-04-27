.PHONY: build test run

build:
	cargo build

run:
	cargo run --release -- examples/config/gain.csd

test:
	cargo test
