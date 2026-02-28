# Suggestions for vision.md and roadmap.md

All suggestions from the pre-step-8-cleanup, step-8-implementation, and step-8-docs/issues.md-cleanup sections have been applied to both docs/design/vision.md and docs/design/roadmap.md as of 2026-02-28.

---

## New Suggestions — Step 8 Gathering Disciplines

These suggestions are based on planning and designing Steps 8.2-8.5 (gathering discipline substeps).

### vision.md Updates

#### 1. PlantCharacteristic and Herbalism types are now implemented
Step 8.2 (Herbalism) has been implemented. The following types now exist in `src/library/types.rs`:
- `PlantCharacteristic` enum: Fragile, Thorny, Aromatic, Bitter, Luminous
- `HerbalismCardEffect { target_characteristics, durability_cost }` — player cards target characteristics and cost durability
- `PlantCard { characteristics, counts }` — enemy cards with typed characteristics
- `HerbalismDef { plant_hand, rewards }` — encounter definition
- `HerbalismEncounterState { round, encounter_card_id, outcome, plant_hand, rewards }`
- `EncounterState::Herbalism(HerbalismEncounterState)`
- `TokenType::HerbalismDurability` (persistent, init 100) and `TokenType::Plant` (reward material)
Update vision.md to reflect these as implemented, not future.

#### 2. Herbalism has two loss conditions (update vision.md)
Vision.md currently describes Herbalism's loss condition as only "0 enemy cards remain (over-harvested)". The implementation adds a second loss condition: HerbalismDurability depletion (each player card has a durability_cost applied immediately on play). Update vision.md to mention both loss conditions.

#### 3. Add new token types for remaining gathering disciplines
When each discipline is implemented, add to the TokenType enum documentation:
- `WoodcuttingDurability`, `TreeHealth`, `Lumber` (Step 8.3)
- `FishingDurability`, `Fish`, `Patience` (Step 8.4)

#### 4. Update "Distinct encounter playstyles" section
The Mining/Woodcutting bullet currently says "focus on discipline wear and extraction" with a note about simplified implementation. As each discipline is built, update this bullet to describe the actual distinct playstyles:
- Mining: damage-vs-durability tradeoff (ore_damage vs durability_prevent)
- Herbalism: card-characteristic matching with durability cost (precision/knowledge)
- Woodcutting: same template as Mining, validates pattern reuse
- Fishing: patience/timing with seeded probability rolls

#### 5. Document the "no draw" enemy pattern (Herbalism)
Herbalism introduces a new enemy behavior: enemy starts with a fixed hand and never draws. This is a meaningful departure from combat (enemy plays and draws) and mining (ore deck draws). Document this as a recognized encounter sub-pattern that future encounter types may reuse.

#### 6. Document the win-by-narrowing pattern (Herbalism)
Herbalism's "win when exactly 1 card left, lose when 0 left" is a new win/loss pattern distinct from HP-depletion (combat/mining) and round-exhaustion (fishing). Vision should acknowledge this pattern as a valid win/loss design.

### roadmap.md Updates

#### 1. Mark Step 8.2 as COMPLETE
Update step 8.2 to include a "— COMPLETE" marker (matching the pattern of step 8.1). Add a summary:
- BREAKING changes: none (additive only)
- New card IDs: 16 (Narrow Herbalism), 17 (Medium Herbalism), 18 (Broad Herbalism), 19 (Meadow Herb encounter)
- Playable acceptance: ✅ Herbalism end-to-end with 3 card types (narrow/medium/broad characteristic targeting), 2 scenario tests (full loop + abort), replay support.

#### 2. Update implementation checklist to include durability_cost
The roadmap 8.2 checklist item for HerbalismCardEffect should note `durability_cost` field (not in original roadmap). Each player card has an immediate durability cost, making durability depletion a second loss condition.

#### 3. Consider a Step 8.6 for gathering balance pass
After all 4 disciplines are implemented (8.1-8.4), a dedicated balance/tuning step could:
- Normalize durability init values (all currently 100 — may need differentiation)
- Balance reward token amounts across disciplines
- Tune encounter card counts and difficulty distributions
- Add cross-discipline scenario tests (e.g., mine then herbalism then combat in sequence)

#### 4. Note the two mechanical templates emerging
Two distinct patterns are forming for gathering encounters:
1. **Damage-vs-durability loop** (Mining, Woodcutting): player deals damage to node HP while node deals durability damage; mutual draw each turn.
2. **Unique mechanic** (Herbalism card-matching, Fishing patience-roll): fundamentally different resolution requiring bespoke logic.
Step 8.5 should consider whether the damage-vs-durability template can be generalized into shared game_state methods vs. keeping each discipline's resolution independent.

### Copilot Instruction Suggestions

#### 1. Update player action list
The copilot instructions reference "Only four player actions exist." The actual list is five: NewGame, EncounterPickEncounter, EncounterPlayCard, EncounterApplyScouting, EncounterAbort.

#### 2. Document the `DeckCounts` shared type
When describing enemy or ore deck card tracking, reference the shared `DeckCounts { deck, hand, discard }` struct.

#### 3. Document the Token-keyed reward pattern
Rewards use `HashMap<Token, i64>` (full Token with lifecycle), not `HashMap<TokenType, i64>`. This should be followed for all future encounter type rewards.

#### 4. Note the inline-computation pattern
When the enemy/ore plays immediately after the player (no intervening state), intermediate values (like durability_prevent) should be passed as function parameters rather than stored on state. This reduces state complexity.

#### 5. Add gathering encounter implementation checklist template
For each new gathering discipline, the standard checklist is:
1. Add new CardKind variant with discipline-specific effect struct
2. Add new EncounterState variant + state struct
3. Add new TokenType variants (durability, encounter-scoped, reward)
4. Add cards and encounter to initialize_library()
5. Init durability in GameState::new()
6. Dispatch in action handler and game_state resolution
7. Update /library/cards?card_kind= filter
8. Update replay_from_log
9. Add scenario test

---

## Contradictions Remaining

### 1. vision.md line ~220: "Conditional: persist until ... Durability hitting 0"
Still uses generic "Durability" rather than discipline-specific names. Minor — the concept is correct but the example could be updated to "MiningDurability hitting 0" for consistency.

### 2. vision.md line ~330: "Durability as a discipline HP pool"
In the "Different token uses" bullet under "Design consequences and examples", still uses generic "Durability". Should be updated to reference discipline-specific durability tokens.

### 3. vision.md line ~465: "discipline Durability ≤ 0"
In the "Win / Loss semantics" section, still uses generic "Durability" in the example. Should reference discipline-specific names.
