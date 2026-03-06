#!/usr/bin/env bash
set -uo pipefail

# Unified check script: runs all validations and reports ALL failures at the end.
# Used by `make check` and as the agent's single validation command.

FAILURES=()

# --- 1. Format (auto-fix + stage) ---
echo "=== cargo fmt (auto-fix) ==="
cargo fmt
FMT_EXIT=$?
if [ $FMT_EXIT -ne 0 ]; then
  FAILURES+=("cargo fmt")
else
  # Stage any formatting changes
  git diff --name-only --diff-filter=M 2>/dev/null | while read -r file; do
    if [[ "$file" == *.rs ]]; then
      git add "$file"
    fi
  done
  echo "  ✓ fmt passed"
fi

# --- 2. Clippy ---
echo ""
echo "=== cargo clippy ==="
cargo clippy --all-targets --all-features -- -D warnings 2>&1
CLIPPY_EXIT=$?
if [ $CLIPPY_EXIT -ne 0 ]; then
  FAILURES+=("cargo clippy")
else
  echo "  ✓ clippy passed"
fi

# --- 3. Build ---
echo ""
echo "=== cargo build ==="
cargo build 2>&1
BUILD_EXIT=$?
if [ $BUILD_EXIT -ne 0 ]; then
  FAILURES+=("cargo build")
else
  echo "  ✓ build passed"
fi

# --- 4. Tests (skip known failures) ---
# Known failures are listed in docs/design/known_failures.md
# We read test names to skip from that file if it exists.
SKIP_ARGS=""
KNOWN_FAILURES_FILE="docs/design/known_failures.md"
if [ -f "$KNOWN_FAILURES_FILE" ]; then
  # Extract test names from lines like "- `test_name`" or "- `test_name` —"
  while IFS= read -r line; do
    test_name=$(echo "$line" | grep -oP '(?<=`)[a-zA-Z_][a-zA-Z0-9_]*(?=`)' | head -1)
    if [ -n "$test_name" ]; then
      SKIP_ARGS="$SKIP_ARGS --skip $test_name"
    fi
  done < <(grep -E '^\s*-\s*`[a-zA-Z_]' "$KNOWN_FAILURES_FILE")
fi

echo ""
echo "=== cargo test ==="
if [ -n "$SKIP_ARGS" ]; then
  echo "  (skipping known failures: see $KNOWN_FAILURES_FILE)"
  # shellcheck disable=SC2086
  cargo test -- $SKIP_ARGS 2>&1
else
  cargo test 2>&1
fi
TEST_EXIT=$?
if [ $TEST_EXIT -ne 0 ]; then
  FAILURES+=("cargo test")
else
  echo "  ✓ tests passed"
fi

# --- 5. Check for unwraps in production code ---
echo ""
echo "=== unwrap check ==="
UNWRAP_COUNT=$(grep -rn "\.unwrap()" src/ --include="*.rs" 2>/dev/null | grep -v "// ok:" | grep -v "#\[cfg(test)\]" | wc -l)
if [ "$UNWRAP_COUNT" -gt 0 ]; then
  echo "  ⚠ Found $UNWRAP_COUNT unwrap() calls in src/ (review these):"
  grep -rn "\.unwrap()" src/ --include="*.rs" | grep -v "// ok:" | grep -v "#\[cfg(test)\]"
  # Warning only, not a failure — existing unwraps may be justified
fi

# --- Summary ---
echo ""
echo "==============================="
if [ ${#FAILURES[@]} -eq 0 ]; then
  echo "✅ All checks passed"
  exit 0
else
  echo "❌ ${#FAILURES[@]} check(s) failed:"
  for f in "${FAILURES[@]}"; do
    echo "  - $f"
  done
  exit 1
fi
