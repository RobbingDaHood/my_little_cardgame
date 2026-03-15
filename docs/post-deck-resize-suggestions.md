# Post-Deck-Resize Suggestions for vision.md and roadmap.md

Generated after implementing: "Resize all decks to 50 D+H"

---

## Suggestions for vision.md

### 1. Add a "Deck Sizing Principles" section under Balancing

The resize work revealed that deck sizes are a first-class balancing lever, not just a cosmetic number. Consider adding to the **Balancing** section:

> **Deck sizing as a balance dial**
> Every deck (player, enemy, encounter pool) is sized to a target of 50 cards (deck + hand combined).
> - Player decks are proportionally scaled to preserve the relative frequency of each card archetype.
> - Enemy sub-decks (ore_deck, fish_deck, interference_deck, etc.) are sized to match the player's pool, so neither side feels thin.
> - Encounter selection pools are sized uniformly so no single discipline dominates the scouting hand.
> All unique card entries are preserved regardless of target size — depth of the card library is never sacrificed for balance.

### 2. Clarify the encounter selection pool model

The vision mentions "Areas as decks" but does not explicitly describe the encounter hand. Consider adding:

> **Encounter selection pool**
> Each discipline contributes a fixed number of encounter cards to a shared scouting pool. Players see up to Foresight (default 3) encounters at a time. The pool is replenished as encounters are consumed. All discipline pools are the same size (currently 50 each) so no discipline is over- or under-represented in the scouting phase.

### 3. Add a note about test configuration isolation

> **Test isolation pattern**
> JSON-driven game initialization (`create_test_client_from_json`) allows tests to use minimal, targeted configurations instead of the full production config. This decouples test correctness from production deck sizes, so changes to balance (like deck resizing) do not break tests that test mechanics, only tests that test specific production values.

---

## Suggestions for roadmap.md

### 1. Add completed milestone: Deck Resize to 50

Under "Implementation updates", add:

```
### Implementation update (deck resize — 50 D+H)
- All player decks, enemy sub-decks, and encounter selection pools sized to 50 D+H.
- Research and interference decks expanded; combat/woodcutting/fishing/herbalism/crafting/mining reduced or expanded proportionally.
- Milestone sub-decks (combat enemy, ore, plant, fish) also sized to 50.
- Test suite refactored: research tests no longer depend on encounter-hand depletion loops; now use isolated JSON configs with CombatInsight pre-seeded.
```

### 2. Add future roadmap item: Dynamic deck sizing

> **Step: Dynamic deck sizing via designer config**
> Currently deck sizes are hardcoded in JSON configs. A future step could expose a `target_deck_size` field in discipline configs so designers can adjust sizing without a JSON diff review. This would also allow difficulty tiers (e.g. a "lite" mode at 25 D+H).

### 3. Update the encounter pool description in roadmap

The current roadmap mentions "Foresight-controlled encounter hands" but does not specify the pool size or the uniformity requirement. Update to:

> - Encounter selection pools are uniformly sized (currently 50 per discipline) so scouting produces a representative cross-section of available encounters.
