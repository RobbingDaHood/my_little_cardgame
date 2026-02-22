# Suggested Improvements to vision.md and roadmap.md

Based on the work done fixing issues.md across two rounds of refactoring.

---

### Round 1 (prior work)

1. **CardDef should declare effects**: The vision mentions cards have types but doesn't specify that card definitions carry declarative effects (token operations with target and amount). Add a note that `CardDef` includes a list of `CardEffect` entries specifying target (self/opponent), token_id, and amount. This is now implemented.

2. **CombatAction is a card play, not a token operation**: The vision should clarify that combat actions are card plays (combatant_id + card_id). Token manipulation (GrantToken, ConsumeToken) happens internally as a result of resolving a card's declared effects — not as separate player-facing actions.

3. **ScoutingParameters replaced by tokens**: The vision's scouting section mentions "preview count, affix bias, pool modifier" as parameters. These are now derived from Foresight tokens rather than stored as an explicit struct. Update the scouting description to say: "Scouting preview count = 1 + Foresight token count. Additional scouting parameters (affix bias, pool modifier) may be derived from other tokens in future steps."

4. **EncounterPhase naming**: The vision doesn't mention specific phase names, but if it does in future, use `NoEncounter`, `Ready`, `InCombat`, `Scouting` (not `Finished` or `PostEncounter`).

5. **HP as tokens**: The vision mentions HP but should clarify that HP is modeled as tokens (`health` and `max_health` in `active_tokens`) rather than as dedicated fields. This aligns with "everything is a token."

### Round 2 (Library-centric refactor)

6. **Library is now implemented**: The vision describes the Library as a canonical catalog with `[library, deck, hand, discard]` exclusive counts. This is now implemented as `Library { cards: Vec<LibraryCard> }` where each `LibraryCard` has `kind: CardKind` and `counts: CardCounts { library, deck, hand, discard }`. The index in the Vec serves as the card ID. Update the vision to note this implementation pattern.

7. **CardKind replaces string-based card_type**: The vision should document the `CardKind` enum with typed payloads: `Attack { effects }`, `Defence { effects }`, `Resource { effects }`, `CombatEncounter { combatant_def }`. This replaces ad-hoc string card types.

8. **Enemy cards are inline, not Library references**: The vision should clarify that enemy card definitions (`EnemyCardDef`) are embedded within the `CombatantDef` of a `CombatEncounter` card, not stored as separate Library entries. Enemies are self-contained.

9. Combat is always player vs. one enemy.

11. **FixedTypeDuration is phase-aware**: The vision should note that `FixedTypeDuration` lifecycles track which encounter phases they count down during, via a `phases: Vec<EncounterPhase>` field.

12. **Two combat systems coexist**: The roadmap should acknowledge that the codebase currently has two combat systems: `src/combat/` (old: HTTP-driven Combat/Unit/States) and `library::combat` (new: deterministic CombatSnapshot/CombatAction). Full unification is future work.

13. **Deck CRUD endpoints removed**: The roadmap should note that explicit deck management endpoints (`/tests/decks/*`, `/tests/cards/*`) have been removed. Card locations are now tracked exclusively by the Library. The remaining old deck types (Deck, DeckCard, CardState) exist only to support the old combat system.

---

### Round 1 (prior work)

1. **Step 6 — CombatAction is now a struct**: The roadmap says "Define CombatState, CombatAction" — update to note that CombatAction is a simple struct `{ is_player, card_index }` (not an enum with DealDamage/GrantToken variants). Effects are resolved from CardDef.

2. **Step 6 — No CombatLog type**: The roadmap mentions "deterministic combat log" — clarify that there is no separate CombatLog type. `simulate_combat` returns `CombatSnapshot` directly. Combat events are recorded in the canonical ActionLog.

3. **Step 7 — EncounterState is minimal**: The roadmap should note that `EncounterState` was removed. The encounter phase is tracked via `EncounterPhase` enum directly.

4. **Step 7 — EncounterFinish is system-driven**: The roadmap mentions "pick, fight, replace, scouting" — add a note that finishing an encounter is a system-driven transition, not a player action.

5. **Step 7 — Startup area deck initialization**: The server now initializes with a starter area deck containing 3 combat encounters. Note this as part of step 7 acceptance.

6. **Step 7 — /combat/simulate moved to /tests/***: The simulate endpoint is now at `POST /tests/combat/simulate` (temporary testing endpoint).

### Round 2 (Library-centric refactor)

7. **Step 1 — Library is the authoritative card catalog**: The roadmap should emphasize that the Library struct is now the single source of truth for card definitions and location counts. `player_data.decks` has been removed. Card state transitions (draw, play, discard, return) happen through Library operations.

8. **Step 6 — CombatSnapshot replaces CombatState**: Update step 6 to reflect the rename and structural changes: explicit `player_tokens` + `enemy` fields instead of `combatants: Vec`, `is_player: bool` on CombatAction instead of `combatant_id`.

9. **Step 6 — Combatant.id removed**: Combat always involves the current encounter enemy; there's no need for combatant IDs.

10. **Step 5 — AreaDeck simplified to Library references**: AreaDeck now stores `Vec<usize>` (Library card indices) instead of full Encounter structs. The `Encounter`, `EncounterState`, `Affix`, and `AffixPipeline` types were removed. Encounter definitions come from the Library's `CombatEncounter` cards. The AreaDeck could be removed in a future step, because it is reduntant. 

11. **New step needed — Unify combat systems**: A future roadmap step should address unifying `src/combat/` (old) with `library::combat` (new). Currently `resolve_card_effects` reads from `player_data.cards` while the Library tracks its own card definitions separately. Insert it as a 7.5 step, in between 7 and 8. 

12. **New step needed — Remove old deck types**: Once the combat systems are unified, remove `Deck`, `DeckCard`, `CardState` from `src/deck/` and `player_data.cards`. The Library fully replaces these. Add that to the step 7.5 above too. 

13. **Test endpoint cleanup**: Deck CRUD endpoints (`/tests/decks/*`, `/tests/cards/*`) have been removed. The remaining test endpoint is `POST /tests/combat` for combat initialization and `POST /tests/library/cards` for test card injection. A future step should migrate combat initialization to the action handler. Add this to the step 7.5 too. 

---

### Round 3 (Step 7.5 + 7.6 implementation, issues.md fixes)

#### Changes to vision.md suggestions

14. **TokenId is now a strongly-typed enum**: The vision mentions tokens by name (Health, Insight, Foresight, etc.) but should document that token identifiers are a closed `TokenId` enum (not strings). Each variant carries lifecycle metadata via `TokenId::lifecycle()`. The current 15 variants are: Health, MaxHealth, Shield, Stamina, Dodge, Mana (combat), Insight, Renown, Refinement, Stability, Foresight, Momentum, Corruption, Exhaustion, Durability (persistent). New tokens require adding an enum variant.

15. **ScopedToEncounter lifecycle deleted**: The vision and codebase previously had a `ScopedToEncounter` token lifecycle. This has been replaced with `FixedTypeDuration { phases, duration }` parameterized by encounter phases. Vision should remove any reference to `ScopedToEncounter` and document `FixedTypeDuration` as the mechanism for phase-aware token expiry.

16. **Combat is unified into GameState**: The vision should clarify that combat state (CombatSnapshot, combat phase, last result) lives in `GameState`, not as a separate HTTP-driven system. The old `src/combat/` module now delegates to `GameState` methods (`start_combat`, `resolve_player_card`, `resolve_enemy_play`, `advance_combat_phase`). There is no longer a separate CombatState type.

17. **Dodge absorption mechanic**: The vision mentions Defence cards but should document the dodge absorption mechanic: when a player or enemy takes damage, dodge tokens are consumed first before health is reduced. This is an important combat interaction not currently described in vision.md.

18. **Resource cards are the draw engine**: The vision should explicitly state that `Resource` cards are the only way to draw additional cards. Each `CardKind::Resource` variant has a `draw_count: u32` field specifying how many random cards are drawn when the resource card is played. This is the core pacing mechanic.

19. **Enemy plays one card from each deck type per turn**: The vision says "enemy resolves its actions according to its card script" but should clarify the current simple behavior: the enemy plays one random card from each of its three deck types (attack, defence, resource) per turn. Card scripts are a future enhancement.

20. **Starting deck composition matters**: The vision should document that starting decks are composed of ~50% resource cards to ensure steady card flow. Current composition: Attack 20 (25%), Defence 20 (25%), Resource 40 (50%), totaling 80 cards.

21. **AreaDeck has deck/hand/discard zones**: The vision describes area decks but should clarify that `AreaDeck` mirrors the player deck model with deck, hand, and discard zones. The hand represents visible/pickable encounters controlled by the Foresight token. Default Foresight is 3.

22. **AbandonCombat action exists**: The vision should mention that players can abandon combat via the `AbandonCombat` action, which clears combat state, records a loss, and returns to the Ready encounter phase. This replaces the need for a future "flee card" at the basic level.

#### Changes to roadmap.md suggestions

14. **Steps 7.5 and 7.6 are now implemented**: The roadmap should mark steps 7.5 and 7.6 as complete. All playable acceptance criteria are met: unified combat system, resource-card driven draws, Foresight-controlled encounter hands, enemy random play, ~50% draw cards in starting decks, and the minimal pick→fight→scouting→pick loop works.

15. **Legacy code fully removed**: The roadmap should note that all legacy deck types (`Deck`, `DeckCard`, `CardState`, `Card`, `Token`), the old `resolve.rs` combat module, and unused helper functions have been removed. The codebase is clean of dead code.

16. **Coverage requirement met**: Step 7.5 mentioned CI failures due to <85% coverage. Coverage is now at 85.86% with comprehensive integration and unit tests. The roadmap should note this is resolved.

17. **FinishScouting is player-driven**: The roadmap should clarify that `FinishScouting` is a player action (not system-driven) that transitions from Scouting → Ready phase. The system only enters Scouting automatically when combat ends with one side at 0 HP.

#### Contradictions found and resolved

1. **Two combat systems → unified**: The codebase had both `src/combat/` (old HTTP-driven) and `library::combat` (new deterministic). These are now unified — `src/combat/` endpoints delegate to `GameState` methods. The old `resolve_card_effects` function has been deleted.

2. **String token IDs → enum**: `Combatant.active_tokens`, `CardEffect.token_id`, etc. all used `String` keys. These are now `TokenId` enum throughout. JSON serialization uses capitalized variant names ("Health", "Shield", etc.).

3. **Deck module was dead code**: `src/deck/mod.rs`, `src/deck/card.rs`, `src/deck/token.rs` contained types (`Deck`, `DeckCard`, `CardState`, `Card`, `Token`, `CardType`) that were no longer referenced anywhere. All deleted.

4. **`player_seed.rs` contained dead helpers**: `derive_subseed`, `snapshot_rng`, `restore_rng_from_snapshot` were never called. Removed.

#### Areas needing future attention

1. **i64 → u64 for token amounts**: issues.md suggested using `u64` instead of `i64` for token amounts. This was deferred because card effects use negative amounts for damage. If adopted, would need saturating subtraction semantics. Consider adding a newtype `TokenAmount(u64)` with damage-aware arithmetic.

2. **CardDef.card_type is still a String**: The `CardDef` type in the combat determinism tests still uses `card_type: String`. This is low priority since `CardDef` is primarily a test type and `CardKind` (enum) is used in the Library.

3. **Scouting is minimal**: Current scouting just returns the encounter card to the area deck. Steps 8+ should expand scouting to generate replacement encounters with affixes as described in the vision.

4. **Enemy AI is random**: Enemies play a random card from each deck. Vision mentions "card scripts" for enemies — this is future work (step 16).

5. **No gathering/crafting yet**: Steps 8-10 (gathering, crafting, research) are not yet implemented. These are the next major features.

6. **Action log replay only handles GrantToken and SetSeed**: `replay_from_log` currently only replays `GrantToken` and `SetSeed` actions. Future steps should expand replay to handle combat, encounter, and card-play actions for full deterministic replay.
