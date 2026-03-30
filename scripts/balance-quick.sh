#!/usr/bin/env bash
# Run balance simulation for a single discipline (or all).
# Usage: scripts/balance-quick.sh [discipline] [--full]
#
# Examples:
#   scripts/balance-quick.sh mining          # mining only, with output
#   scripts/balance-quick.sh combat --full   # combat only, full output
#   scripts/balance-quick.sh                 # all disciplines (same as make balance-check)
#
# Disciplines: combat, mining, herbalism, woodcutting, fishing, all

set -euo pipefail

DISCIPLINE="${1:-all}"
EXTRA_ARGS="${2:-}"

# Map discipline names to test filter patterns
case "$DISCIPLINE" in
  combat)      FILTER="combat_balance_simulation" ;;
  mining)      FILTER="mining_balance_simulation" ;;
  herbalism)   FILTER="herbalism_balance_simulation" ;;
  woodcutting) FILTER="woodcutting_balance_simulation" ;;
  fishing)     FILTER="fishing_balance_simulation" ;;
  all)         FILTER="" ;;
  *)
    echo "Unknown discipline: $DISCIPLINE"
    echo "Valid: combat, mining, herbalism, woodcutting, fishing, all"
    exit 1
    ;;
esac

# Build the cargo test command
CMD="cargo test --features simulation --test balance"
if [ -n "$FILTER" ]; then
  CMD="$CMD $FILTER"
fi
CMD="$CMD -- --nocapture"

echo "=== Balance: $DISCIPLINE ==="
echo "Running: $CMD"
echo ""

# Time the run
START=$(date +%s)
eval "$CMD"
END=$(date +%s)
ELAPSED=$((END - START))
echo ""
echo "=== Completed in ${ELAPSED}s ==="
