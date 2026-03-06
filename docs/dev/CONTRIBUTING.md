# Contributing

Thank you for contributing to my_little_cardgame.

Developer expectations
- Keep changes small and focused.
- Run formatting and lints before committing: `cargo fmt` and `cargo clippy --all-targets --all-features -- -D warnings`.
- **All tests must pass before pushing code.** Run `make check` (or `cargo test`) and ensure zero failures. Never accept or commit known test failures. If a test fails, fix the test or the production code before committing. If in doubt about whether to fix a test or the production code, ask the repository owner.
- Maintain test coverage and fix regressions; CI enforces an 85% coverage threshold.
- Avoid `unwrap()`/`expect()` in production code; prefer Result propagation or handle poisoned mutexes.
- ActionLog concurrency: when recording actions from async contexts prefer `append_async` (or clone the Arc<ActionLog> and call `append_async` after dropping async locks) to avoid blocking async executors; see repository docs for rationale.

Pre-commit hooks
- Install pre-commit hooks:

```bash
make install-hooks
# or
./scripts/install-hooks.sh
```

CI
- The repository includes a GitHub Actions workflow that runs formatting, clippy, tests and coverage on each PR.

How to run tests locally
- Run full test suite: `cargo test`
- Run single test: `cargo test <test_name_substring>`
- Run tests with visible output: `cargo test -- --nocapture`

Reporting issues
- Create GitHub issues for bugs or proposed changes. Small fixes should include tests and documentation updates.

Thank you!
