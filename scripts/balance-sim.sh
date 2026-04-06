#!/usr/bin/env bash
# Run balance simulation for a single discipline or all disciplines.
#
# Usage:
#   scripts/balance-sim.sh <discipline> [--quick|--explore|--full]
#
# Disciplines: combat, mining, herbalism, woodcutting, fishing, all
#
# Modes (override game/encounter counts without recompiling):
#   --quick    1 game, 10 encounters  (~2-5s)   directional signal
#   --explore  3 games, 20 encounters (~8-15s)  rough signal
#   --full     (default) use test-defined values (~40-500s) final validation
#
# Examples:
#   scripts/balance-sim.sh woodcutting --quick
#   scripts/balance-sim.sh mining --explore
#   scripts/balance-sim.sh all
#   SIM_GAMES=5 SIM_ENCOUNTERS=30 scripts/balance-sim.sh combat

set -euo pipefail

DISCIPLINE="${1:-}"
MODE="${2:-full}"

if [[ -z "$DISCIPLINE" ]]; then
    echo "Usage: scripts/balance-sim.sh <discipline> [--quick|--explore|--full]"
    echo ""
    echo "Disciplines: combat, mining, herbalism, woodcutting, fishing, all"
    echo "Modes: --quick (1×10), --explore (3×20), --full (test defaults)"
    exit 1
fi

# Map discipline to test name
case "$DISCIPLINE" in
    combat)     TEST_NAME="combat_balance_simulation" ;;
    mining)     TEST_NAME="mining_balance_simulation" ;;
    herbalism)  TEST_NAME="herbalism_balance_simulation" ;;
    woodcutting) TEST_NAME="woodcutting_balance_simulation" ;;
    fishing)    TEST_NAME="fishing_balance_simulation" ;;
    all)        TEST_NAME="" ;;
    *)
        echo "Unknown discipline: $DISCIPLINE"
        echo "Valid: combat, mining, herbalism, woodcutting, fishing, all"
        exit 1
        ;;
esac

# Apply mode overrides (only if not already set via env vars)
case "$MODE" in
    --quick)
        export SIM_GAMES="${SIM_GAMES:-1}"
        export SIM_ENCOUNTERS="${SIM_ENCOUNTERS:-10}"
        echo "Quick mode: ${SIM_GAMES} games × ${SIM_ENCOUNTERS} encounters"
        ;;
    --explore)
        export SIM_GAMES="${SIM_GAMES:-3}"
        export SIM_ENCOUNTERS="${SIM_ENCOUNTERS:-20}"
        echo "Explore mode: ${SIM_GAMES} games × ${SIM_ENCOUNTERS} encounters"
        ;;
    --full)
        echo "Full mode: using test-defined values"
        ;;
    *)
        echo "Unknown mode: $MODE (use --quick, --explore, or --full)"
        exit 1
        ;;
esac

echo "Running $DISCIPLINE balance simulation..."
echo ""

if [[ "$DISCIPLINE" == "all" ]]; then
    cargo test --features simulation --test balance -- --nocapture
else
    cargo test --features simulation --test balance "$TEST_NAME" -- --nocapture
fi
