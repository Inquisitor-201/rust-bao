env:
	rustup target add aarch64-unknown-none

build:
	cargo build --release

.PHONY: env build