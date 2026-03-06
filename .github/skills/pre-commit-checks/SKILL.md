---
name: pre-commit-checks
description: Run all required pre-commit checks for my_little_cardgame. Use this before every commit to verify the code is ready.
---

Run every step below in order before committing. Stop and report immediately if any step fails — do not commit while any step is failing.

## Steps

### 1. Check formatting
```bash
cargo fmt -- --check
```
If this fails, run `cargo fmt` to fix it, then re-run the check.

### 2. Lint with Clippy (zero warnings policy)
```bash
cargo clippy --all-targets --all-features -- -D warnings
```
Fix every warning before proceeding. No unwrap() calls should be added in production code — prefer Result propagation.

### 3. Build
```bash
cargo build
```
Confirms the project compiles cleanly.

### 4. Run full test suite
```bash
cargo test
```
All tests must pass. Aim for at least 90% test coverage before committing.

### 5. Check for unwraps in production code
```bash
grep -rn "\.unwrap()" src/ --include="*.rs"
```
Any new unwrap() in `src/` (excluding test modules) must be justified or replaced with proper error handling.

## Commit message rules

- Always include the co-author trailer at the end of every commit message:
  ```
  Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
  ```
- If the commit contains breaking changes (API, data format, struct layout), prefix the summary line with `BREAKING:` and list what changed in the commit body.
- Keep commits small and isolated — each commit must pass all checks above on its own.
