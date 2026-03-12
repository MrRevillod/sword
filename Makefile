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
	cargo test --workspace

test-log:
	cargo test --workspace -- --nocapture

build:
	cargo build --workspace --all-features

release:
	cargo build --workspace --all-features --release
