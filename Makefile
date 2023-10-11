all: build-test

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

docker:
	docker stop zhus_sip_stack || true
	docker rm zhus_sip_stack || true
	docker rmi zhus_sip_stack_img || true
	docker build -t zhus_sip_stack_img -f Dockerfile.local .
	docker run -d --name zhus_sip_stack zhus_sip_stack_img