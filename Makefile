.PHONY: coverage

coverage:
	rustup component add llvm-tools-preview
	cargo install --locked cargo-llvm-cov || true
	cargo llvm-cov --workspace --lcov --output-path target/lcov.info --fail-under-lines 85
