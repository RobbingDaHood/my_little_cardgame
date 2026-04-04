#!/usr/bin/env bash
# Dump actual API JSON formats for a discipline's cards, effects, and encounters.
# Use this BEFORE writing or modifying any driver/strategy code to verify
# the real JSON structure matches your assumptions.
#
# Usage:
#   scripts/balance-api-inspect.sh herbalism
#   scripts/balance-api-inspect.sh mining
#   scripts/balance-api-inspect.sh combat
#
# This runs a tiny Rust test that starts the game server in-process,
# triggers one encounter for the given discipline, and dumps the JSON
# from key API endpoints. The output shows exact field names, nesting,
# and tagged enum formats — the #1 source of driver bugs.

set -euo pipefail

DISCIPLINE="${1:?Usage: $0 <discipline>}"

echo "═══════════════════════════════════════════"
echo "  API Format Inspector: ${DISCIPLINE}"
echo "═══════════════════════════════════════════"
echo ""

# Run the inspector test for this discipline
cargo test --features simulation --test balance "api_inspect_${DISCIPLINE}" -- --nocapture 2>&1 | \
    grep -v "^running\|^test result\|^$" | \
    sed 's/^test /\n✓ test /'

echo ""
echo "═══════════════════════════════════════════"
echo "  Compare these field names with your"
echo "  driver.rs extraction code!"
echo "═══════════════════════════════════════════"
