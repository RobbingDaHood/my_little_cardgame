#!/bin/bash
# Example API requests for My Little Card Game
# Start the server with `cargo run` before running these examples

BASE_URL="http://localhost:8000"

echo "=== My Little Card Game API Examples ==="
echo

echo "1. List all cards:"
curl -s "${BASE_URL}/cards" | jq '.'
echo

echo "2. Create a new Attack card:"
CARD_RESPONSE=$(curl -s -X POST "${BASE_URL}/cards" \
  -H "Content-Type: application/json" \
  -d '{
    "card_type_id": 1,
    "card_type": "Attack",
    "effects": [],
    "costs": [],
    "count": 10
  }')
CARD_LOCATION=$(echo "$CARD_RESPONSE" | grep -i location || echo "/cards/3")
echo "Created card at: $CARD_LOCATION"
echo

echo "3. List all decks:"
curl -s "${BASE_URL}/decks" | jq '.'
echo

echo "4. Create a new deck:"
DECK_RESPONSE=$(curl -s -X POST "${BASE_URL}/decks" \
  -H "Content-Type: application/json" \
  -d '{
    "contains_card_types": ["Attack", "Defence"]
  }')
DECK_LOCATION=$(echo "$DECK_RESPONSE" | grep -i location || echo "/decks/3")
echo "Created deck at: $DECK_LOCATION"
echo

echo "5. Get combat status:"
curl -s "${BASE_URL}/combat" | jq '.'
echo

echo "6. Initialize combat:"
curl -s -X POST "${BASE_URL}/combat"
echo

echo "7. Get updated combat status:"
curl -s "${BASE_URL}/combat" | jq '.'
echo

echo "=== Validation Examples ==="
echo

echo "8. Try creating card with zero count (should fail):"
curl -s -X POST "${BASE_URL}/cards" \
  -H "Content-Type: application/json" \
  -d '{
    "card_type_id": 1,
    "card_type": "Attack",
    "effects": [],
    "costs": [],
    "count": 0
  }' | jq '.'
echo

echo "9. Try creating deck with no card types (should fail):"
curl -s -X POST "${BASE_URL}/decks" \
  -H "Content-Type: application/json" \
  -d '{
    "contains_card_types": []
  }' | jq '.'
echo

echo "=== Examples complete ==="
