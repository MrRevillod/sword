fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

nursery:
    cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::pedantic -D clippy::nursery

test entity="":
    cargo test {{entity}} --workspace --all-features --quiet

test-log entity="":
    cargo test {{entity}} -- --nocapture

example NAME="basic_web_server":
    cd examples && cargo run --example {{NAME}}

build:
    cargo build --workspace --all-targets

build-release:
    cargo build --workspace --release --all-targets
