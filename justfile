run:
	cargo run

watch:
	DEBUG_LOG=1 cargo watch --ignore data/ -x run

build:
	cargo build --locked

release:
	cargo build --locked --release

deploy: release
	DEBUG_LOG=1 ./target/release/oshi-api

ci:
	cargo check
	cargo test
	cargo fmt --check
	cargo clippy -- --deny warnings
	@echo "✅ All checks passed"
