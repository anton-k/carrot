.PHONY: build test run

build:
	cargo build

run:
	cargo run --release -- examples/config/toggle.yaml

test:
	cargo test
