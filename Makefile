.PHONY: clean fmt lint machete nursery test

clean:
	cargo clean --workspace

fmt:
	cargo fmt --all

lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

machete:
	cargo machete

nursery:
	cargo clippy --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery

test:
	cargo test --workspace --exclude grpc-controllers
	cargo check -p sword --no-default-features --features grpc-controllers
	cargo check -p grpc-controllers

test-log:
	cargo test --workspace --exclude grpc-controllers -- --nocapture

build:
	cargo build --workspace --all-features

release:
	cargo build --workspace --all-features --release
