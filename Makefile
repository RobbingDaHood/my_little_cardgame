.PHONY: coverage check balance-check balance-combat balance-mining balance-herbalism balance-woodcutting balance-fishing

check:
	bash scripts/check_all.sh

balance-check:
	cargo test --features simulation --test balance -- --nocapture

balance-combat:
	bash scripts/balance-quick.sh combat

balance-mining:
	bash scripts/balance-quick.sh mining

balance-herbalism:
	bash scripts/balance-quick.sh herbalism

balance-woodcutting:
	bash scripts/balance-quick.sh woodcutting

balance-fishing:
	bash scripts/balance-quick.sh fishing

coverage:
	rustup component add llvm-tools-preview
	cargo install --locked cargo-llvm-cov || true
	cargo llvm-cov --workspace --lcov --output-path target/lcov.info --fail-under-lines 85

install-hooks:
	@command -v pre-commit >/dev/null 2>&1 || (echo "pre-commit not found; run 'pip install --user pre-commit' and ensure ~/.local/bin is in PATH" && exit 1)
	./scripts/install-hooks.sh
