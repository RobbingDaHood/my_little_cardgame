#!/usr/bin/env bash
set -euo pipefail

if ! command -v rustfmt >/dev/null 2>&1; then
  echo "rustfmt not found. Install with 'rustup component add rustfmt'." >&2
  exit 1
fi

echo "Running cargo fmt (auto-fix)..."
cargo fmt

# Stage any formatting changes so they're included in the commit
git diff --name-only --diff-filter=M | while read -r file; do
  if [[ "$file" == *.rs ]]; then
    git add "$file"
  fi
done
