.PHONY: build test lint fmt clean dev

build:
	cargo build

test:
	cargo test

lint:
	cargo clippy -- -D warnings

fmt:
	cargo fmt -- --check

clean:
	cargo clean

dev: fmt lint test build
