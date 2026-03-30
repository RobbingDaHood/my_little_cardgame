.PHONY: coverage check balance-check balance-quick

check:
	bash scripts/check_all.sh

balance-check:
	cargo test --features simulation --test balance -- --nocapture

# Quick single-discipline balance sim (explore mode). Usage: make balance-quick D=woodcutting
balance-quick:
	@test -n "$(D)" || (echo "Usage: make balance-quick D=<discipline>"; echo "Disciplines: combat, mining, herbalism, woodcutting, fishing, all"; exit 1)
	bash scripts/balance-sim.sh $(D) --explore

coverage:
	rustup component add llvm-tools-preview
	cargo install --locked cargo-llvm-cov || true
	cargo llvm-cov --workspace --lcov --output-path target/lcov.info --fail-under-lines 85

install-hooks:
	@command -v pre-commit >/dev/null 2>&1 || (echo "pre-commit not found; run 'pip install --user pre-commit' and ensure ~/.local/bin is in PATH" && exit 1)
	./scripts/install-hooks.sh
