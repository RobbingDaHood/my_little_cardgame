#!/usr/bin/env bash
set -euo pipefail

# Run the project's coverage target (Makefile target 'coverage') if available; fall back to cargo llvm-cov
if grep -q "^coverage:" Makefile 2>/dev/null; then
  echo "Running make coverage (enforces 85% threshold)..."
  make coverage
else
  echo "Running cargo llvm-cov (enforces 85% threshold)..."
  cargo llvm-cov --workspace --lcov --output-path target/lcov.info --fail-under-lines 85
fi
