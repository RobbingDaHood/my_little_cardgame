#!/usr/bin/env bash
# Run balance simulation for a specific discipline or all disciplines.
# Usage:
#   scripts/balance-sim.sh                   # run all disciplines
#   scripts/balance-sim.sh herbalism         # run herbalism only
#   scripts/balance-sim.sh combat mining     # run combat and mining
#
# Options:
#   --quick   Use reduced simulation (fewer games) for faster feedback

set -euo pipefail

QUICK=false
DISCIPLINES=()

for arg in "$@"; do
    case "$arg" in
        --quick) QUICK=true ;;
        *) DISCIPLINES+=("$arg") ;;
    esac
done

# Build the test filter
if [ ${#DISCIPLINES[@]} -eq 0 ]; then
    FILTER="balance_simulation"
else
    # Join discipline names with | for regex matching
    FILTER=$(printf "%s_balance_simulation\|" "${DISCIPLINES[@]}")
    FILTER="${FILTER%\\|}"  # remove trailing \|
fi

echo "═══════════════════════════════════════════"
echo "  Balance Simulation"
echo "  Filter: ${DISCIPLINES[*]:-all disciplines}"
if $QUICK; then
    echo "  Mode: quick (reduced sample size)"
fi
echo "═══════════════════════════════════════════"

# Run the simulation, extract the JSON report from stdout
cargo test --features simulation --test balance "$FILTER" -- --nocapture 2>&1 | \
    awk '
    /^{/,/^}/ { json=1; print; next }
    /^test .* \.\.\. (ok|FAILED)/ { print }
    /^test result:/ { print }
    ' | head -500

echo ""
echo "═══════════════════════════════════════════"
echo "  Done"
echo "═══════════════════════════════════════════"
