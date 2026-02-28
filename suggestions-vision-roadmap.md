# Suggestions for vision.md and roadmap.md

All suggestions from the pre-step-8-cleanup, step-8-implementation, and step-8-docs/issues.md-cleanup sections have been applied to both docs/design/vision.md and docs/design/roadmap.md as of 2026-02-28.

---

## New Suggestions — Step 8 Gathering Disciplines

These suggestions are based on planning and designing Steps 8.2-8.5 (gathering discipline substeps).

### vision.md Updates

#### 1. Add Herbalism characteristic system to token/type definitions
When Step 8.3 (Herbalism) is implemented, add `PlantCharacteristic` enum (Fragile, Thorny, Aromatic, Bitter, Luminous) to the types documentation. This is a new pattern: enemy cards with typed characteristics rather than HP/damage mechanics. Document how characteristic matching works (player card targets characteristics, removes all enemy cards sharing at least one match).

#### 2. Add new token types for gathering disciplines
When each discipline is implemented, add to the TokenType enum documentation:
- `WoodcuttingDurability`, `TreeHealth`, `Lumber` (Step 8.2)
- `HerbalismDurability`, `Plant` (Step 8.3)
- `FishingDurability`, `Fish`, `Patience` (Step 8.4)

#### 3. Update "Distinct encounter playstyles" section
The Mining/Woodcutting bullet currently says "focus on discipline wear and extraction" with a note about simplified implementation. As each discipline is built, update this bullet to describe the actual distinct playstyles:
- Mining: damage-vs-durability tradeoff (ore_damage vs durability_prevent)
- Woodcutting: same template, validates pattern reuse
- Herbalism: card-characteristic matching (precision/knowledge)
- Fishing: patience/timing with seeded probability rolls

#### 4. Document the "no draw" enemy pattern (Herbalism)
Herbalism introduces a new enemy behavior: enemy starts with a fixed hand and never draws. This is a meaningful departure from combat (enemy plays and draws) and mining (ore deck draws). Document this as a recognized encounter sub-pattern that future encounter types may reuse.

#### 5. Document the win-by-narrowing pattern (Herbalism)
Herbalism's "win when exactly 1 card left, lose when 0 left" is a new win/loss pattern distinct from HP-depletion (combat/mining) and round-exhaustion (fishing). Vision should acknowledge this pattern as a valid win/loss design.

### roadmap.md Updates

#### 1. Consider a Step 8.6 for gathering balance pass
After all 4 disciplines are implemented (8.1-8.4), a dedicated balance/tuning step could:
- Normalize durability init values (all currently 100 — may need differentiation)
- Balance reward token amounts across disciplines
- Tune encounter card counts and difficulty distributions
- Add cross-discipline scenario tests (e.g., mine then woodcut then combat in sequence)

#### 2. Track new CardKind variants needed
Each discipline adds a new CardKind variant. The roadmap should note the growing pattern:
- Attack, Defence, Resource, Mining (existing)
- Woodcutting (8.2), Herbalism (8.3), Fishing (8.4)
This affects the card_kind filter endpoint, replay_from_log dispatch, and encounter start dispatch. A checklist pattern has been established in the substeps.

#### 3. Note the two mechanical templates emerging
Two distinct patterns are forming for gathering encounters:
1. **Damage-vs-durability loop** (Mining, Woodcutting): player deals damage to node HP while node deals durability damage; mutual draw each turn.
2. **Unique mechanic** (Herbalism card-matching, Fishing patience-roll): fundamentally different resolution requiring bespoke logic.
Step 8.5 should consider whether the damage-vs-durability template can be generalized into shared game_state methods vs. keeping each discipline's resolution independent.

### Copilot Instruction Suggestions

#### 1. Update player action list
The copilot instructions reference "Only four player actions exist." Update to five, adding `EncounterAbort`.

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
