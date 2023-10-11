all: test build

fmt:
	cargo fmt --all -- --check

clippy:
	cargo clippy --all -- -D warnings

build-test: fmt clippy
	cargo clean
	cargo test
	cargo build
	cargo build --release
	cargo build --release --target x86_64-unknown-linux-gnu
	cargo build --release --target x86_64-unknown-linux-musl
	ls -lh target/debug
	ls -lh target/release
	ls -lh target/x86_64-unknown-linux-gnu/release
	ls -lh target/x86_64-unknown-linux-musl/release

build:
	cargo clean
	cargo test
	cargo build
	cargo build --release
	cargo build --release --target x86_64-unknown-linux-gnu
	cargo build --release --target x86_64-unknown-linux-musl
	ls -lh target/debug
	ls -lh target/release
	ls -lh target/x86_64-unknown-linux-gnu/release
	ls -lh target/x86_64-unknown-linux-musl/release