# Suggestions for vision.md and roadmap.md

## Changes made in this cleanup (pre-step-8-cleanup branch)

These changes should be reflected in both documents:

### 1. CardEffect struct deleted — cards reference effects by ID only

**Current vision.md describes:**
> CardEffect uses a `kind` field with two variants: `ChangeTokens { target, token_type, amount }` for token manipulation, and `DrawCards { amount }` for card draw.

**Actual implementation now:**
- The `CardEffect` struct no longer exists.
- Cards (Attack, Defence, Resource) store `effect_ids: Vec<usize>` referencing library entries.
- `PlayerCardEffect` and `EnemyCardEffect` variants hold `kind: CardEffectKind` and `lifecycle: TokenLifecycle` directly — they ARE the effect definitions.
- `EnemyCardDef.effects` is now `effect_ids: Vec<usize>`.

**Suggested update for vision.md:** Replace all references to `CardEffect { kind, lifecycle, card_effect_id }` with a description of the pure ID reference model. Cards store `effect_ids: Vec<usize>`; the actual effect data lives on `PlayerCardEffect`/`EnemyCardEffect` library entries.

### 2. DrawCards is now per-deck-type, not a single amount

**Current vision.md describes:**
> `CardEffectKind::DrawCards { amount }` ... Starting decks draw 2 cards per resource play.

**Actual implementation now:**
```rust
DrawCards { attack: u32, defence: u32, resource: u32 }
```
- Initial draw effect: 1 attack, 1 defence, 2 resource (total 4 cards, not "2 cards").
- Player draws from each deck type separately with discard recycling per type.
- Enemy draws from all three enemy deck types per the specified amounts.

**Suggested update for vision.md:** Replace `DrawCards { amount }` with the per-type variant. Update the "draw 2 cards per resource play" to "draw 1 attack, 1 defence, 2 resource per resource play." Clarify that both player and enemy draws happen per deck type.

**Suggested update for roadmap.md:** The line "DrawCards amount is 2 per resource play" is now outdated. Update to reflect per-deck-type draws with the new counts.

### 3. CardDef struct deleted — it was legacy

**Current roadmap.md mentions:**
> CardDef → LibraryCard transition

**Actual implementation now:**
- `CardDef` has been fully deleted. It was only used by the old combat simulation system (`resolve_combat_tick`, `simulate_combat`).
- The `/tests/combat/simulate` endpoint has been removed.
- All combat resolution uses `GameState::resolve_player_card()` and `resolve_enemy_play()`.

**Suggested update for roadmap.md:** Add a note under completed work that CardDef and the old simulation system have been fully removed. The combat simulation endpoint no longer exists.

### 4. encounter.rs state machine deleted

**Current roadmap.md describes:**
> Step 7 encounter loop with `EncounterAction` enum and state machine.

**Actual implementation now:**
- `EncounterAction` enum has been deleted.
- `encounter.rs` module has been deleted.
- `combat.rs` module (library) has been deleted (was empty stub).
- The encounter state is managed directly by `action/mod.rs` via `GameState.encounter_state.phase`.

**Suggested update for vision.md:** Remove any references to `EncounterAction` as a type. The encounter actions are now handled by `PlayerActions` enum in `action/mod.rs`.

### 5. Serde tag renamed from "kind" to "card_kind"

**Breaking API change:** The JSON field discriminator for `CardKind` variants changed from `"kind"` to `"card_kind"` due to a conflict with the `PlayerCardEffect { kind, ... }` field name. Any external API consumers need to update their JSON parsing.

**Suggested update for vision.md:** Note the serde tag name `"card_kind"` for CardKind serialization.

---

## Contradictions found

### 1. vision.md says "draw 2 cards" but implementation draws 4 (1+1+2)
- The per-type draw mechanic means the total number of cards drawn per resource play is now 4, not 2. Vision.md should be updated to reflect this.

### 2. roadmap.md references `library::combat` but the module is deleted
- Line 112: "Unify the two combat implementations (src/combat/ old HTTP-driven Combat/Unit/States and library::combat deterministic CombatSnapshot/CombatAction)"
- The `library::combat` module no longer exists. This milestone is effectively complete — there is only one combat system now.

### 3. vision.md references CardEffect as a struct, but it's deleted
- Multiple lines in vision.md describe CardEffect as a struct with `kind`, `lifecycle`, and `card_effect_id` fields. None of these exist anymore.

### 4. roadmap.md "balance parameters" section is stale
- "DrawCards amount is 2 per resource play (via CardEffectKind::DrawCards { amount: 2 })" — this type signature and value are both outdated.

---

## Areas for improvement

### vision.md
1. **Card effect model:** Rewrite the "CardEffect" section to describe the simplified reference model where cards store `effect_ids` and the actual effect data lives on library entries.
2. **DrawCards section:** Update to describe per-deck-type draws with the struct `{ attack, defence, resource }`.
3. **Enemy deck draws:** Currently says "Resource cards draw their DrawCards amount (via CardEffectKind::DrawCards) of cards for each of the three enemy deck types." This is now correct in spirit but the type description is stale.
4. **Future deck types:** Vision lists Mining, Fabrication, Provisioning, etc. as future deck types. When these are added, the DrawCards variant will need to be extended. Consider whether DrawCards should use a HashMap<CardType, u32> instead of named fields to support future extensibility.

### roadmap.md
1. **Step numbering:** The jump from 7.7 to 8 suggests the sub-steps of 7 are complete. Add a clear "Step 7 COMPLETE" marker.
2. **Completed work summary:** Add a summary of what was done in the pre-step-8-cleanup (CardEffect simplification, per-type draws, dead code removal).
3. **Combat unification:** Mark the combat unification milestone as fully complete — there is only one combat system now (GameState resolution methods).
4. **Test strategy:** Mention that `encounter_loop_e2e.rs` and `combat_determinism.rs` have been removed because they tested deleted production code. All scenario coverage is now in `scenario_tests.rs`.
