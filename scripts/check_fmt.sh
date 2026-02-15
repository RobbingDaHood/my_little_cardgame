#!/usr/bin/env bash
set -euo pipefail

if ! command -v rustfmt >/dev/null 2>&1; then
  echo "rustfmt not found. Install with 'rustup component add rustfmt'." >&2
  exit 1
fi

echo "Running cargo fmt -- --check..."
cargo fmt -- --check
