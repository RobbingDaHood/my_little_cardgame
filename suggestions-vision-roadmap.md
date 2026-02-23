# Suggestions for vision.md and roadmap.md

Observations gathered during implementation of all docs/issues.md items and roadmap step 7.7.

---

## Suggested changes to roadmap.md

### 1. Update replay system notes (Post-7.6 cleanup section)
The note says "replay_from_log does not yet fully replay player actions." This is now implemented. Update to:
> `replay_from_log` now replays player actions (SetSeed, DrawEncounter, PlayCard, ApplyScouting) in addition to legacy token entries. Combined with the initial seed, the action log is sufficient to reconstruct the full game state for the core loop.

### 2. Add post-implementation update for completed work
```
### Post-7.7 implementation (2026-02-23)
- All issues from docs/issues.md resolved:
  - Issue 9: Removed unused `effects` field from EncounterPlayCard
  - Issue 7: Removed with_default_lifecycle; all tokens PersistentCounter except Dodge (FixedTypeDuration to Defence phase); CardEffect now carries explicit lifecycle
  - Issue 2: Removed lifecycle from TokenRegistryEntry (now only id + cap)
  - Issue 4: Token maps serialize as compact JSON objects (e.g., {"Health": 20}); backward-compatible deserialization
  - Issue 5: Renamed CombatSnapshot → CombatState
  - Issue 6: Enemy decks track deck/hand/discard counts; hand shuffle at combat start; play from hand only
  - Issue 8: /tokens endpoint returns full TokenRegistryEntry objects
  - Issue 1: replay_from_log handles SetSeed, DrawEncounter, PlayCard, ApplyScouting
- Step 7.7 implemented: PlayerCardEffect and EnemyCardEffect CardKind variants; card_effect_id references; validation; GET /library/card-effects endpoint
- Card IDs shifted: 0-3 player CardEffect cards, 4-7 enemy CardEffect cards, 8+ action/encounter cards
```

### 3. Mark Step 3 replay acceptance as partially met
The playable acceptance for Step 3 says "a replay test reconstructs state from seed + action log." This now works for the core game loop.

### 4. Step 7.7 needs a "Playable acceptance" section
The current 7.7 description lists requirements but no acceptance criteria. Suggest adding:
> Playable acceptance: Library contains both player and enemy CardEffect decks; all card effects on player/enemy cards reference valid CardEffect deck entries (validated at initialization); GET /library/card-effects returns both decks.

---

## Suggested changes to vision.md

### 1. Update enemy deck model
Line 44 says "Enemy turns play one random card matching the current CombatPhase." Should mention:
> Enemy decks use EnemyCardCounts { deck, hand, discard } to track card locations. At combat start, hands are shuffled. Enemies play from hand only; played cards go to discard. Resource DrawCards effects draw from deck to hand with discard recycling.

### 2. Update CombatState naming
vision.md still references "CombatSnapshot" in places. Consistently use "CombatState" throughout.

### 3. Document EncounterPlayCard signature
Line 50 says `EncounterPlayCard { card_id }` — this is now correct (effects field removed). No change needed, but verify this stays in sync.

### 4. Document CardEffect decks concept
Add a new section or expand the Library description to mention:
> The Library contains two CardEffect "decks": PlayerCardEffect cards (for future research mechanics) and EnemyCardEffect cards (for future scouting mechanics). Every CardEffect on action/enemy cards references a CardEffect deck entry via card_effect_id. API responses show full effect values, not references.

### 5. Document token lifecycle changes
The vision should reflect that all tokens are now PersistentCounter except Dodge (FixedTypeDuration to Defence phase), and that CardEffect carries an explicit lifecycle field.

---

## Contradictions and areas for improvement

1. **Ambiguity in enemy resource draws**: issues.md says "take a random card from 'the deck'" when resource card triggers. Unclear whether this means the same deck type or any enemy deck. Currently draws from same deck type only.

2. **No attack/defence deck replenishment**: After initial hand is played, attack and defence decks have no draw mechanism (only resource cards trigger draws). Should there be a round-based draw-to-hand mechanic?

3. **EnemyCardEffect deck usage unclear**: Step 7.7 says enemy CardEffect deck "will be used during scouting" and player CardEffect deck "during research." These future uses should be described more concretely — what does "used during scouting" mean? Are these decks drawn from, or are they just catalogs?

4. **Token map serialization and vision.md**: The compact JSON format ({"Health": 20}) is not documented in vision.md. Since API format is part of the contract, consider documenting it.

5. **Card ID stability**: With CardEffect cards now at indices 0-7, any future additions to the CardEffect decks will shift all other card IDs. Consider using stable IDs (e.g., HashMap instead of Vec) or documenting this fragility.

6. **DrawCards as TokenType**: DrawCards is listed under TokenType but isn't a real token — it's an effect-only trigger. Consider making it a distinct mechanism rather than overloading the token system.
