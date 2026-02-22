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
