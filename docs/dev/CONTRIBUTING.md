# Contributing

Thank you for contributing to my_little_cardgame.

Developer expectations
- Keep changes small and focused.
- Run `make check` before committing — it runs formatting, clippy, build, tests, and coverage (≥80% threshold) in one pass.
- **All tests must pass before pushing code.** Never accept or commit known test failures. If a test fails, fix the test or the production code before committing. If in doubt, ask the repository owner.
- Maintain test coverage and fix regressions; CI enforces an 80% coverage threshold.
- Avoid `unwrap()`/`expect()` in production code; prefer Result propagation or handle poisoned mutexes.
- ActionLog concurrency: when recording actions from async contexts prefer `append_async` (or clone the Arc<ActionLog> and call `append_async` after dropping async locks) to avoid blocking async executors.

Documentation expectations
- **OpenAPI doc comments**: When adding or changing endpoints, update `///` doc comments on handler functions explaining *why* a player or designer would use the endpoint.
- **Self-documenting endpoints**: When modifying game mechanics, card effects, encounter types, or disciplines, update the relevant `/docs/*` endpoint source:
  - `src/docs/tutorial.rs` — new-player walkthrough
  - `src/docs/hints.rs` — per-discipline strategies and tips
  - `src/docs/designer.rs` — designer reference for encounter/card/token authoring
- **README.md**: When adding new endpoints, add them to the API table.
- **Examples**: Keep `docs/examples/api_examples.sh` working with current endpoints and payloads.
- After documentation changes, run the server and spot-check `/swagger/`, `/docs/tutorial`, `/docs/hints`, and `/docs/designer`.

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
- Run full validation: `make check`
- Run full test suite: `cargo test`
- Run single test: `cargo test <test_name_substring>`
- Run tests with visible output: `cargo test -- --nocapture`
- Run coverage only: `cargo llvm-cov --workspace --fail-under-lines 80`

Reporting issues
- Create GitHub issues for bugs or proposed changes. Small fixes should include tests and documentation updates.

Thank you!
