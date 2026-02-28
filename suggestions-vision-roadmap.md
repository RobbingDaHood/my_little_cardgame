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
- `WoodcuttingDurability`, `Lumber` (Step 8.3) — note: TreeHealth removed since Woodcutting no longer uses an enemy/tree HP mechanic
- `FishingDurability`, `Fish`, `Patience` (Step 8.4)

#### 4. Update "Distinct encounter playstyles" section — DONE
The Mining/Woodcutting bullet has been split into separate bullets describing actual distinct playstyles:
- Mining: damage-vs-durability tradeoff (ore_damage vs durability_prevent)
- Woodcutting: rhythm-based pattern matching (ChopType + chop_value, poker-inspired patterns)
- Herbalism: card-characteristic matching with durability cost (precision/knowledge)
- Fishing: patience/timing with seeded probability rolls

#### 5. Document the "no draw" enemy pattern (Herbalism)
Herbalism introduces a new enemy behavior: enemy starts with a fixed hand and never draws. This is a meaningful departure from combat (enemy plays and draws) and mining (ore deck draws). Document this as a recognized encounter sub-pattern that future encounter types may reuse.

#### 6. Document the win-by-narrowing pattern (Herbalism)
Herbalism's "win when exactly 1 card left, lose when 0 left" is a new win/loss pattern distinct from HP-depletion (combat/mining) and round-exhaustion (fishing). Vision should acknowledge this pattern as a valid win/loss design.

#### 7. Document the pattern-evaluation win pattern (Woodcutting)
Woodcutting introduces a third win/loss pattern: the player always "wins" (encounter completes) but the quality of the win (Lumber reward) depends on the best pattern formed from the 8 played cards. This is distinct from binary win/lose outcomes. Vision should document this as a "degree-of-success" encounter pattern.

#### 8. Document the "no enemy deck" pattern (Woodcutting)
Woodcutting has no enemy/node deck at all — the encounter is purely about the player's card choices. This is a new encounter sub-pattern where challenge comes from hand management and pattern construction rather than opponent interaction.

### roadmap.md Updates

#### 1. Mark Step 8.2 as COMPLETE
Update step 8.2 to include a "— COMPLETE" marker (matching the pattern of step 8.1). Add a summary:
- BREAKING changes: none (additive only)
- New card IDs: 16 (Narrow Herbalism), 17 (Medium Herbalism), 18 (Broad Herbalism), 19 (Meadow Herb encounter)
- Playable acceptance: ✅ Herbalism end-to-end with 3 card types (narrow/medium/broad characteristic targeting), 2 scenario tests (full loop + abort), replay support.
- Note: All herbalism cards now cost 1 durability. Plant hand is randomized at encounter start using seeded RNG.

#### 2. Update implementation checklist to include durability_cost
The roadmap 8.2 checklist item for HerbalismCardEffect should note `durability_cost` field (not in original roadmap). Each player card has an immediate durability cost, making durability depletion a second loss condition.

#### 3. Consider a Step 8.6 for gathering balance pass
After all 4 disciplines are implemented (8.1-8.4), a dedicated balance/tuning step could:
- Normalize durability init values (all currently 100 — may need differentiation)
- Balance reward token amounts across disciplines
- Tune encounter card counts and difficulty distributions
- Add cross-discipline scenario tests (e.g., mine then herbalism then combat in sequence)

#### 4. Note the three mechanical templates emerging
Three distinct patterns are forming for gathering encounters:
1. **Damage-vs-durability loop** (Mining): player deals damage to node HP while node deals durability damage; mutual draw each turn.
2. **Unique mechanic — card matching** (Herbalism): card-characteristic matching to narrow the enemy hand; no enemy draws.
3. **Unique mechanic — pattern building** (Woodcutting): play cards to build poker-like patterns; no enemy deck at all; degree-of-success rather than binary win/lose.
4. **Unique mechanic — patience/probability** (Fishing): seeded probability rolls with decreasing patience.
Step 8.5 should consider whether any of these can share infrastructure or whether each discipline's resolution should remain fully independent.

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

#### 6. Document the shuffle-at-encounter-start pattern
All encounter types should randomize their internal decks/hands at encounter start using the seeded RNG. This was added to herbalism (plant_shuffle_hand) matching the existing combat (enemy_shuffle_hand) and mining (ore_shuffle_hand) patterns. Add this as a standard convention.

#### 7. Document the range system (future, once step 9.1 is implemented)
Once CardEffects use the range system (min-min/max-min/min-max/max-max), update the copilot instructions to describe how numeric values on cards are generated and resolved. This affects how new cards and effects should be defined.

---

## Contradictions and Areas of Improvement

### Contradictions Remaining

#### 1. vision.md line ~220: "Conditional: persist until ... Durability hitting 0"
Still uses generic "Durability" rather than discipline-specific names. Minor — the concept is correct but the example could be updated to "MiningDurability hitting 0" for consistency.

#### 2. vision.md line ~330: "Durability as a discipline HP pool"
In the "Different token uses" bullet under "Design consequences and examples", still uses generic "Durability". Should be updated to reference discipline-specific durability tokens.

#### 3. vision.md line ~465: "discipline Durability ≤ 0"
In the "Win / Loss semantics" section, still uses generic "Durability" in the example. Should reference discipline-specific names.

#### 4. Roadmap 8.3 previously described Woodcutting as "same template as Mining"
This was corrected in this session. The roadmap and vision now describe Woodcutting as rhythm-based pattern matching. However, vision.md may still have residual references to the old Mining-clone template in other sections — search for "chop_damage" or "splinter_prevent" and remove.

**Update (2026-02-28 — Step 8.3 implementation):** Step 8.3 was implemented using the Mining-clone template (damage-vs-durability loop with chop_damage/splinter_prevent) as described in the original roadmap 8.3 spec. The pattern-matching/rhythm-based Woodcutting variant described in the vision was NOT implemented — that is deferred to Step 8.5 (Refined gathering encounters). The current implementation validates the EncounterState pattern is reusable, which was the stated goal of 8.3.

#### 5. vision.md Encounter templates section numbering
The encounter templates section numbers Woodcutting as "2)" which was originally "same template as Mining". This has been updated, but the section numbering and flow should be reviewed for consistency after all 4 disciplines are implemented.

### Areas of Improvement

#### 1. No documentation of the 4 encounter win/loss patterns
The vision currently doesn't have an explicit section cataloging the different win/loss patterns. There are now 4 distinct patterns:
- HP depletion (Combat, Mining)
- Card narrowing (Herbalism: exactly 1 remaining)
- Degree of success / pattern evaluation (Woodcutting: always wins, reward varies)
- Probability/patience (Fishing: seeded roll or patience expiry)
A new section in vision.md enumerating these patterns would help when designing future encounter types.

#### 2. Stamina is mentioned but not well-defined across disciplines
Stamina appears as a combat token, but steps 9.2 and 9.3 introduce it as a cross-discipline resource (cost for cards, recovered by rest). Vision.md should have a clear description of Stamina's cross-discipline role.

#### 3. The roadmap doesn't describe how rest encounters interact with the encounter deck
Step 9.3 adds rest encounters to the encounter deck, but the vision doesn't describe how different encounter types should be distributed in the area deck or how that distribution evolves.

#### 4. No clear description of when numbers get bumped 100x
Step 9.1 mentions bumping numbers by 100x, but the vision and other roadmap steps still reference small numbers (e.g., "3 shield", "5 damage", "durability 100"). After implementing 9.1, all references in vision.md and roadmap.md to specific numeric values will need updating.

#### 5. Insight is mentioned in TokenType but not yet in CardEffectKind
The Insight token type exists, but there's no CardEffectKind for granting it. Step 8.5 now describes this, but vision.md should document the Insight card effect pattern as a general mechanism (any card can have an effect that grants discipline-specific tokens).

---

## New Suggestions — Step 8.3 Woodcutting Implementation (2026-02-28)

### roadmap.md Updates

#### 1. Mark Step 8.3 as COMPLETE
Add implementation notes to the "Step 8 implementation updates" section:
- Step 8.3 (Woodcutting) implemented following Mining template as specified.
- New card IDs: 20 (Aggressive Woodcutting), 21 (Balanced Woodcutting), 22 (Protective Woodcutting), 23 (Oak Tree encounter).
- New tokens: WoodcuttingDurability (persistent, init 100), TreeHealth (encounter-scoped), Lumber (reward material).
- 2 scenario tests added (full loop + abort).
- Playable acceptance: ✅ Woodcutting end-to-end with chop_damage/splinter_prevent tradeoff, produces Lumber tokens, EncounterAbort supported.

#### 2. Confirm 3 disciplines now validate EncounterState reusability
Mining, Herbalism, and Woodcutting all use the same EncounterState enum pattern with discipline-specific state structs. The pattern is confirmed reusable: each discipline adds a new variant to EncounterState, EncounterKind, and CardKind, then plugs into the existing action dispatch and replay infrastructure.

### vision.md Updates

#### 1. Update Woodcutting token documentation
vision.md line ~29 lists WoodcuttingDurability and Lumber but notes "TreeHealth removed since Woodcutting no longer uses an enemy/tree HP mechanic". This is now outdated — Step 8.3 DOES use TreeHealth as an encounter-scoped token (tree HP that the player chops down). Update vision.md to include TreeHealth in the Woodcutting token list.

#### 2. Clarify simplified vs refined Woodcutting
vision.md describes Woodcutting in two ways: (a) the simplified Mining-clone template (step 8.3) and (b) the full rhythm/pattern-matching version (step 8.5). These are now both partially documented. Add a note clarifying that the current implementation is the simplified version and the pattern-matching version is the Step 8.5 refinement target.

### Contradictions

#### 6. suggestions-vision-roadmap.md items 7 and 8 are now contradicted
Items "#### 7. Document the pattern-evaluation win pattern (Woodcutting)" and "#### 8. Document the 'no enemy deck' pattern (Woodcutting)" describe the refined Woodcutting (step 8.5) which has NOT been implemented. The current 8.3 implementation uses the standard damage-vs-durability loop WITH an enemy tree deck. These items should be moved to a "Step 8.5 future" section to avoid confusion.

