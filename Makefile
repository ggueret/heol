.PHONY: fmt lint test check build clean

fmt:
	cargo fmt

lint:
	cargo clippy --all-targets --all-features -- -D warnings

test:
	cargo test --all-features

check: fmt
	cargo fmt --check
	cargo clippy --all-targets --all-features -- -D warnings
	cargo test --all-features

build:
	cargo build --release

clean:
	cargo clean
