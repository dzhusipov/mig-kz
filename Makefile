all: test build

test:
	cargo test

build:
	cargo clean
	cargo build
	cargo build --release
	ls -lh target/debug/
	ls -lh target/release/