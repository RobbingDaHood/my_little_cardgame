#!/bin/bash
# Example API walkthrough for My Little Card Game
# Start the server with `cargo run` before running these examples.
# Requires: curl, jq
#
# This script demonstrates a complete gameplay loop:
#   NewGame → pick encounter → play cards → conclude → scout → metrics

set -euo pipefail
BASE_URL="http://localhost:8000"

echo "=== My Little Card Game — Full Gameplay Walkthrough ==="
echo

# ── Step 1: Read the tutorial ──────────────────────────────────────
echo "1. Read the new-player tutorial:"
curl -s "${BASE_URL}/docs/tutorial" | jq '.steps | length' | xargs -I{} echo "   Tutorial has {} steps"
echo

# ── Step 2: Start a new game (deterministic seed) ─────────────────
echo "2. Start a new game with seed 42:"
curl -s -X POST "${BASE_URL}/action" \
  -H "Content-Type: application/json" \
  -d '{"action_type": "NewGame", "seed": 42}' | jq '.message // .'
echo

# ── Step 3: Check token balances ──────────────────────────────────
echo "3. Check starting token balances:"
curl -s "${BASE_URL}/player/tokens" | jq 'to_entries | map(select(.value != 0)) | from_entries'
echo

# ── Step 4: See available actions ─────────────────────────────────
echo "4. Available actions (encounter choices):"
ACTIONS=$(curl -s "${BASE_URL}/actions/possible")
echo "$ACTIONS" | jq '[.[] | .action_type] | unique'
echo

# ── Step 5: Pick a non-combat encounter ───────────────────────────
echo "5. Pick first available encounter card:"
CARD_ID=$(echo "$ACTIONS" | jq '[.[] | select(.action_type == "EncounterPickEncounter")][0].card_id')
echo "   Picking encounter card_id=${CARD_ID}"
curl -s -X POST "${BASE_URL}/action" \
  -H "Content-Type: application/json" \
  -d "{\"action_type\": \"EncounterPickEncounter\", \"card_id\": ${CARD_ID}}" | jq '.message // .'
echo

# ── Step 6: Check encounter state ─────────────────────────────────
echo "6. Current encounter state:"
ENCOUNTER=$(curl -s "${BASE_URL}/encounter")
echo "$ENCOUNTER" | jq '{outcome: .outcome, phase: .encounter_state.phase // "N/A"}'
echo

# ── Step 7: Play cards until encounter resolves ───────────────────
echo "7. Playing cards until encounter resolves..."
for i in $(seq 1 20); do
  PLAY_ACTIONS=$(curl -s "${BASE_URL}/actions/possible")
  PLAY_CARD=$(echo "$PLAY_ACTIONS" | jq '[.[] | select(.action_type == "EncounterPlayCard")][0].card_id // empty')
  
  if [ -z "$PLAY_CARD" ]; then
    echo "   No more cards to play after ${i} rounds"
    break
  fi
  
  curl -s -X POST "${BASE_URL}/action" \
    -H "Content-Type: application/json" \
    -d "{\"action_type\": \"EncounterPlayCard\", \"card_id\": ${PLAY_CARD}}" > /dev/null
  
  OUTCOME=$(curl -s "${BASE_URL}/encounter" | jq -r '.outcome')
  if [ "$OUTCOME" != "Undecided" ]; then
    echo "   Encounter resolved after ${i} card(s): ${OUTCOME}"
    break
  fi
done
echo

# ── Step 8: Conclude the encounter ────────────────────────────────
echo "8. Conclude encounter:"
curl -s -X POST "${BASE_URL}/action" \
  -H "Content-Type: application/json" \
  -d '{"action_type": "EncounterConcludeEncounter"}' | jq '.message // .'
echo

# ── Step 9: Apply scouting (accept defaults) ──────────────────────
echo "9. Apply scouting (accept default picks):"
curl -s -X POST "${BASE_URL}/action" \
  -H "Content-Type: application/json" \
  -d '{"action_type": "EncounterApplyScouting", "card_ids": []}' | jq '.message // .'
echo

# ── Step 10: Check encounter results ──────────────────────────────
echo "10. Encounter history:"
curl -s "${BASE_URL}/encounter/results" | jq '.'
echo

# ── Step 11: Check session metrics ────────────────────────────────
echo "11. Session metrics after one encounter:"
curl -s "${BASE_URL}/metrics" | jq '{
  total_encounters: .total_encounters,
  total_wins: .total_wins,
  disciplines: [.disciplines[] | select(.encounters_played > 0) | {discipline: .discipline, played: .encounters_played, won: .encounters_won}]
}'
echo

# ── Step 12: Check token balances after encounter ─────────────────
echo "12. Token balances after encounter:"
curl -s "${BASE_URL}/player/tokens" | jq 'to_entries | map(select(.value != 0)) | from_entries'
echo

# ── Step 13: View action log for replay ───────────────────────────
echo "13. Action log (for deterministic replay):"
curl -s "${BASE_URL}/actions/log" | jq 'length' | xargs -I{} echo "   {} actions recorded"
echo

# ── Bonus: Check hints and designer guide ─────────────────────────
echo "=== Documentation Endpoints ==="
echo
echo "Hints — disciplines covered:"
curl -s "${BASE_URL}/docs/hints" | jq '[.disciplines[].discipline]'
echo
echo "Designer guide — sections:"
curl -s "${BASE_URL}/docs/designer" | jq 'keys'
echo

echo "=== Walkthrough complete ==="
echo "Run the server (cargo run) and try these commands interactively!"
echo "Interactive Swagger UI: ${BASE_URL}/swagger/"
