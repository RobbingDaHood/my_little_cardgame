# Suggestions for vision.md and roadmap.md

## Changes made in this cleanup (pre-step-8-cleanup branch)

These changes should be reflected in both documents:

### 1. CardEffect struct deleted — cards reference effects by ID only

**Current vision.md describes:**
> CardEffect uses a `kind` field with two variants: `ChangeTokens { target, token_type, amount }` for token manipulation, and `DrawCards { amount }` for card draw.

**Actual implementation now:**
- The `CardEffect` struct no longer exists.
- Cards (Attack, Defence, Resource) store `effect_ids: Vec<usize>` referencing library entries.
- `PlayerCardEffect` and `EnemyCardEffect` variants hold `kind: CardEffectKind` directly — they ARE the effect definitions.
- `EnemyCardDef.effects` is now `effect_ids: Vec<usize>`.
- `lifecycle` field has been removed from both `PlayerCardEffect` and `EnemyCardEffect` — lifecycle is solely on the `TokenType`.

**Suggested update for vision.md:** Replace all references to `CardEffect { kind, lifecycle, card_effect_id }` with a description of the pure ID reference model. Cards store `effect_ids: Vec<usize>`; the actual effect data lives on `PlayerCardEffect`/`EnemyCardEffect` library entries. Remove any mention of lifecycle on card effects.

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
- The encounter state is managed directly by `action/mod.rs` via `GameState.encounter_phase`.

**Suggested update for vision.md:** Remove any references to `EncounterAction` as a type. The encounter actions are now handled by `PlayerActions` enum in `action/mod.rs`.

### 5. Serde tag renamed from "kind" to "card_kind"

**Breaking API change:** The JSON field discriminator for `CardKind` variants changed from `"kind"` to `"card_kind"` due to a conflict with the `PlayerCardEffect { kind, ... }` field name. Any external API consumers need to update their JSON parsing.

**Suggested update for vision.md:** Note the serde tag name `"card_kind"` for CardKind serialization.

### 6. TokenRegistry removed — tokens created directly from TokenType

**Actual implementation now:**
- `TokenRegistry` struct and `src/library/registry.rs` have been deleted.
- `GameState.registry` field removed.
- `/tokens` endpoint removed.
- Tokens are created directly from `TokenType` using `Token::persistent(token_type)`.
- `GameState.token_balances` is the sole source of truth for token state.

**Suggested update for vision.md:** Remove references to TokenRegistry as a runtime struct. Tokens are created from `TokenType` directly. The canonical token definitions live in `TokenType` enum, not in a registry.

**Suggested update for roadmap.md:** Step 4 references "canonical token registry" — clarify that token definitions are now encoded in the `TokenType` enum and `Token` struct, not in a separate registry data structure.

### 7. ActionPayload simplified to match PlayerActions

**Actual implementation now:**
- `ActionPayload` has been reduced to 4 variants: `SetSeed`, `DrawEncounter`, `PlayCard`, `ApplyScouting`.
- Internal operations (GrantToken, ConsumeToken, ExpireToken, RngDraw, RngSnapshot, ReplaceEncounter, ConsumEntryCost) have been removed.
- The action log now only records player-initiated actions.

**Suggested update for roadmap.md:** The action log design section should note that the log records only player actions (this is already stated in the alignment requirements but contradicted by older step descriptions that mention logging token lifecycle events).

### 8. ActionEntry simplified to just seq + payload

**Actual implementation now:**
- `ActionEntry` now contains only `seq: usize` and `payload: ActionPayload`.
- Removed fields: `action_type`, `timestamp`, `actor`, `request_id`, `version`.
- The `/actions/log` endpoint no longer supports `?action_type=` or `?since=` filters.
- `seq` is derived from the entry's index in the action log vector.

**Suggested update for roadmap.md:** Step 3 describes ActionEntry with rich metadata (timestamp, actor, etc.). Update to reflect the simplified design where entries are just sequential payloads.

### 9. EncounterState wrapper removed — EncounterPhase used directly

**Actual implementation now:**
- `EncounterState` wrapper struct has been deleted.
- `GameState` uses `encounter_phase: EncounterPhase` directly.
- `EncounterPhase` variants: `NoEncounter`, `Combat`, `Scouting` (removed `Ready`, renamed `InCombat`→`Combat`).

**Suggested update for vision.md:** Replace any mention of `EncounterState` with `EncounterPhase`. Document the three phases: NoEncounter (waiting for player to pick), Combat (active combat), Scouting (post-combat scouting).

### 10. CombatState.player_turn removed

**Actual implementation now:**
- `player_turn` field removed from `CombatState`.
- Turn control is implicit: the player always acts first, then the system auto-resolves enemy play and advances combat phase.

**Suggested update for vision.md:** If vision.md mentions turn-based alternation, clarify that the player acts and the system auto-resolves enemy response within the same action.

### 11. CombatantDef.initial_tokens changed to u64

**Actual implementation now:**
- `CombatantDef.initial_tokens` uses `HashMap<Token, u64>` (unsigned).
- Runtime `token_balances` still uses `i64` for signed arithmetic (damage can reduce below zero during calculation).
- Custom serde module `token_map_serde_u64` handles u64 serialization.

**Suggested update for vision.md:** Clarify that initial token values are always non-negative (u64), while runtime balances may temporarily go negative during calculations (i64).

### 12. combat_results expanded to Vec<CombatOutcome>

**Actual implementation now:**
- `GameState.last_combat_result: Option<CombatOutcome>` replaced with `combat_results: Vec<CombatOutcome>`.
- Each completed combat pushes its outcome to the vector.
- Endpoint changed from `/combat/result` to `/combat/results` (returns full history).

**Suggested update for roadmap.md:** Note that combat results are now a historical list, not just the last result. This supports replay verification and progression tracking.

### 13. Area deck module fully removed

**Actual implementation now:**
- `src/area_deck/` module has been completely deleted (mod.rs, endpoints.rs, scouting.rs).
- `AreaDeck` struct deleted — was only used in tests.
- `ScoutingParams` struct deleted — was only used with AreaDeck.
- `/area` and `/area/encounters` endpoints removed.
- Encounter cards are now accessed via `/library/cards?location=Hand&card_kind=Encounter`.
- The library cards endpoint returns `LibraryCardWithId` (includes card ID/index) with optional `?location=` and `?card_kind=` filters.

**Suggested update for vision.md:** Remove references to AreaDeck as a runtime struct. Encounter cards are managed through Library card location tracking (Deck/Hand/Discard) using the same mechanism as all other card types. The "everything is a deck" principle is now implemented purely through Library card counts.

**Suggested update for roadmap.md:** Step 5 describes AreaDeck as a separate data structure. Update to reflect that encounter card management is now part of the Library's general card location system. ScoutingParams will need to be re-implemented when step 11 (post-encounter scouting choices) is reached, but as part of the Library/GameState system rather than as a separate module.

### 14. PlayerData renamed to RandomGeneratorWrapper

**Actual implementation now:**
- `PlayerData` struct renamed to `RandomGeneratorWrapper` in `src/player_data.rs`.
- The struct is purely an RNG wrapper (no player-specific data beyond the random generator).

**Suggested update for vision.md:** If PlayerData is mentioned, update to RandomGeneratorWrapper or describe it as the RNG wrapper.

### 15. draw_player_cards_of_kind now draws random cards

**Actual implementation now:**
- `draw_player_cards_of_kind` accepts `&mut rng` parameter and uses it to pick a random index from drawable cards.
- Previously always drew the first card (index 0), making draws deterministic but not random.

**Suggested update for vision.md:** Ensure the card draw description mentions that draws are random (seeded) from the available pool, not sequential.

### 16. CardKind used in allowed_card_kind

**Actual implementation now:**
- `CombatPhase::allowed_card_kind()` returns `fn(&CardKind) -> bool` (a typed predicate).
- Previously returned `&'static str` which was compared against string representations.

**Suggested update for vision.md:** When describing combat phases and allowed card types, note that the check is type-safe via CardKind enum matching.

### 17. CardLocation enum added

**Actual implementation now:**
- New `CardLocation` enum: `Library`, `Deck`, `Hand`, `Discard`.
- Used as a query filter on `/library/cards?location=Hand`.
- Enables location-based filtering of any card type through a single endpoint.

**Suggested update for vision.md:** Document CardLocation as part of the card state model. Each card in the library exists in one of four locations: Library (canonical definition), Deck (shuffled, not yet drawn), Hand (drawn, available to play), Discard (played/used).

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

### 5. roadmap.md references TokenRegistry but it's deleted
- Step 4 mentions "canonical token registry" and "implement token types, caps/decay rules, and lifecycle metadata."
- TokenRegistry has been deleted. Token definitions are now in the `TokenType` enum. The `Token` struct carries `token_type` and `lifecycle` fields.

### 6. roadmap.md describes AreaDeck as a separate data structure but it's deleted
- Step 5 says "Implement AreaDeck, encounter consumption, replacement-generation." 
- AreaDeck has been deleted. Encounter cards use the Library's general card location tracking system (deck/hand/discard counts per card).

### 7. roadmap.md alignment requirements mention "token registry" lifecycle declarations
- "Tokens declare lifecycle semantics in the token registry" — the token registry no longer exists. Lifecycle is declared on the `Token` struct directly.

### 8. roadmap.md step 3 describes ActionEntry with rich metadata
- Step 3 says "append-only, chronologically ordered ActionLog that records player action metadata."
- ActionEntry now contains only `seq` and `payload`. No timestamp, actor, or other metadata. The action log records player actions, but the "metadata" concept has been simplified.

### 9. roadmap.md step 7.7 cleanup mentions "CardEffect now carries explicit lifecycle"
- Post-7.7 cleanup note says "CardEffect now carries explicit lifecycle."
- Lifecycle has been removed from card effects entirely. Only TokenType carries lifecycle.

### 10. roadmap.md post-7.6 cleanup mentions "/tokens endpoint returns full TokenRegistryEntry objects"
- The `/tokens` endpoint has been removed along with `TokenRegistryEntry`.

---

## Areas for improvement

### vision.md
1. **Card effect model:** Rewrite the "CardEffect" section to describe the simplified reference model where cards store `effect_ids` and the actual effect data lives on library entries (PlayerCardEffect/EnemyCardEffect).
2. **DrawCards section:** Update to describe per-deck-type draws with the struct `{ attack, defence, resource }`.
3. **Enemy deck draws:** Currently says "Resource cards draw their DrawCards amount (via CardEffectKind::DrawCards) of cards for each of the three enemy deck types." This is now correct in spirit but the type description is stale.
4. **Future deck types:** Vision lists Mining, Fabrication, Provisioning, etc. as future deck types. When these are added, the DrawCards variant will need to be extended. Consider whether DrawCards should use a HashMap<CardType, u32> instead of named fields to support future extensibility.
5. **Token lifecycle model:** Clarify that lifecycle is solely on `TokenType`/`Token`, not on card effects. Card effects simply reference a `TokenType` and the lifecycle comes from the token definition.
6. **Encounter state model:** Replace EncounterState references with EncounterPhase. Document the three phases (NoEncounter, Combat, Scouting) and the auto-advance behavior.
7. **Card location model:** Add a section describing CardLocation (Library, Deck, Hand, Discard) and how all card types use the same location tracking system.
8. **AreaDeck references:** Remove AreaDeck as a named struct. Encounter cards are tracked via Library card counts like all other card types.
9. **Token definitions:** Replace "token registry" language with direct TokenType enum descriptions. The canonical token list is the TokenType enum.

### roadmap.md
1. **Step numbering:** The jump from 7.7 to 8 suggests the sub-steps of 7 are complete. Add a clear "Step 7 COMPLETE" marker.
2. **Completed work summary:** Add a "Pre-step-8 cleanup" section summarizing all changes from this branch: lifecycle removal from card effects, TokenRegistry removal, ActionPayload/ActionEntry simplification, EncounterState removal, AreaDeck removal, CombatState.player_turn removal, combat_results expansion, CardLocation addition, u64 initial_tokens, random card draw fix.
3. **Combat unification:** Mark the combat unification milestone as fully complete — there is only one combat system now (GameState resolution methods).
4. **Test strategy:** Mention that multiple test files have been removed (encounter_loop_e2e.rs, combat_determinism.rs, library_integration.rs, proptest_sequences.rs, proptest_replay.rs, replay_determinism.rs, area_deck_integration.rs, area_deck_e2e.rs) because they tested deleted production code. All scenario coverage is now in `scenario_tests.rs`.
5. **Alignment requirements:** Update the single-mutator-endpoint section — current player actions are NewGame, EncounterPickEncounter, EncounterPlayCard, EncounterApplyScouting (already correct, but verify against code).
6. **Step 5 (AreaDeck):** Mark the structural AreaDeck work as superseded by Library card location tracking. The scouting/replacement mechanics from step 5 are deferred to step 11.
7. **Step 4 (Token lifecycle):** Update to reflect that token definitions are in the `TokenType` enum, not a separate registry. Remove mentions of TokenRegistryEntry.
8. **ScoutingParams note:** Add a note that ScoutingParams was deleted in the cleanup and will need to be re-implemented as part of step 11 (post-encounter scouting choices) when that work begins.
9. **Action log clarity:** The alignment requirements correctly state "action log records only player actions." Ensure step 3's description matches this (remove any mention of logging internal token operations).

---

## Step 8 Implementation — New Suggestions

These suggestions are based on implementing Step 8 (mining gathering encounters):

### vision.md Updates

#### 1. Update Combat/Encounter terminology
The vision doc still refers to `CombatState` and `CombatOutcome`. These have been renamed:
- `CombatState` → `EncounterState` (an enum with `Combat(CombatEncounterState)` and `Mining(MiningEncounterState)` variants)
- `CombatOutcome` → `EncounterOutcome` (variants: `PlayerWon`, `PlayerLost`, `Undecided` — `EnemyWon` renamed to `PlayerLost`)
- `current_combat` → `current_encounter`, `combat_results` → `encounter_results`

#### 2. Document the EncounterState enum pattern
Add a section explaining the encounter dispatch pattern:
```rust
pub enum EncounterState {
    Combat(CombatEncounterState),
    Mining(MiningEncounterState),
    // future: Herbalism, Woodcutting, Crafting, etc.
}
```
Each encounter type has its own struct because mechanics differ fundamentally. Combat has 3 decks + phases; mining has a single deck with no phases. This pattern should guide future encounter type implementations.

#### 3. Document Mining encounter mechanics
The vision doc describes gathering disciplines at a high level. Now that Mining is implemented, add concrete mechanics:
- **Single-deck resolution**: Player Mining deck (cards with `ore_damage` + `durability_prevent` tradeoff). Ore has OreDeck with cards dealing 0-3 durability damage (skewed low: ~30% zero, ~40% one, ~20% two, ~10% three).
- **Turn flow**: Player plays → ore_damage reduces ore HP, durability_prevent stored → ore plays random card → effective durability damage = raw - prevent → both draw → check end.
- **Win**: Ore HP ≤ 0 → grant material tokens (Ore: 10).
- **Lose**: Player MiningDurability ≤ 0 → encounter ends as lost, no penalties applied.
- **No phases**: Unlike combat's Defending → Attacking → Resourcing cycle, mining is one action per turn.

#### 4. Clarify endpoint renaming
`/combat` → `/encounter`, `/combat/results` → `/encounter/results`. All encounter types share these endpoints. The response JSON includes `encounter_state_type` discriminator field (`"Combat"` or `"Mining"`).

#### 5. Add Mining to card kind enumeration
`CardKind::Mining { mining_effect: MiningCardEffect }` is now alongside Attack, Defence, Resource. Mining effects are inline (`ore_damage: i64, durability_prevent: i64`) rather than using effect_id references.

#### 6. Document new token types (UPDATED)
- `TokenType::Ore`: Persistent material token granted on mining success. First material token; Woodcutting/Herbalism will add their own.
- `TokenType::MiningDurability` (renamed from `Durability`): Initialized to 100 at game start via `GameState::new()`. Functions as mining HP. Persists across encounters and decreases over time. When it reaches 0, the player loses the current mining encounter. It is NOT re-initialized per encounter.
- `TokenType::OreHealth`: Encounter-scoped token tracking ore node HP. Stored in `MiningEncounterState.ore_tokens` (a `HashMap<Token, i64>`). Replaces the old `ore_hp`/`ore_max_hp` fields.

#### 7. Update EncounterPhase (UPDATED)
`EncounterPhase::Combat` and `EncounterPhase::Gathering` have been merged into `EncounterPhase::InEncounter`. The phases are now: `NoEncounter`, `InEncounter`, `Scouting`. Both combat and mining encounters use the `InEncounter` phase.

### roadmap.md Updates

#### 1. Mark Step 8 as partially complete (UPDATED)
- ✅ Mining gathering discipline: playable end-to-end with 3 card types (Aggressive ore_damage=5/prevent=0, Balanced 3/2, Protective 1/3)
- ✅ Iron Ore encounter: ore HP 15, 20 ore cards weighted low, rewards Ore:10 (Token-keyed), no failure penalties
- ✅ EncounterState enum pattern established as template for future encounter types
- ✅ BREAKING: `/combat` → `/encounter` endpoint rename, `CombatState` → `EncounterState`
- ✅ Mining scenario tests demonstrating full gameplay loop
- ✅ Replay system updated for mining encounters
- ✅ docs/issues.md cleanup: 10 issues resolved (see below)
- ⬜ Woodcutting and Herbalism disciplines (follow same pattern)
- ⬜ Rations/Durability cross-discipline consumption (Step 15)

#### 2. Simplify future gathering discipline steps
With the EncounterState enum as a template, adding new disciplines is mechanical:
1. Add new CardKind variant (e.g., `Woodcutting { effect: WoodcuttingEffect }`)
2. Add new EncounterState variant + state struct
3. Add cards to `initialize_library()`
4. Dispatch in action handler and game_state resolution
5. Update `/library/cards?card_kind=` filter
6. Update `replay_from_log`
7. Add scenario tests

Consider making each discipline a separate sub-step of Step 8.

#### 3. Note the replay coverage pattern
`replay_from_log` dispatches by encounter type for `DrawEncounter` and by EncounterState variant for `PlayCard`. Each new encounter type must extend replay. Add this as an explicit checklist item for new encounter type steps.

#### 4. Card_kind filter maintenance
The `/library/cards?card_kind=` endpoint was extended to support `Mining`. Each new card kind needs to be added to this filter. Consider auto-deriving from the CardKind enum in a future cleanup to avoid manual updates.

#### 5. Library card IDs
Current library card ID assignments (for reference in future steps):
- 0-3: PlayerCardEffect entries (damage, shield, stamina, draw)
- 4-7: EnemyCardEffect entries
- 8: Attack card, 9: Defence card, 10: Resource card
- 11: Combat encounter (Gnome)
- 12: Mining card (Aggressive), 13: Mining card (Balanced), 14: Mining card (Protective)
- 15: Mining encounter (Iron Ore)

---

## Step 8 docs/issues.md Cleanup — New Suggestions

These suggestions are based on implementing all 10 issues from docs/issues.md:

### vision.md Updates

#### 1. Generalized deck counts — `DeckCounts` replaces `EnemyCardCounts` and `OreCardCounts`
**Current vision.md describes:** `EnemyCardCounts { deck, hand, discard }` for enemy card tracking.
**Actual implementation now:** A single `DeckCounts { deck, hand, discard }` struct is used by both enemy and ore card tracking. The old `EnemyCardCounts` and `OreCardCounts` types have been deleted.
**Suggested update:** Replace all mentions of `EnemyCardCounts` with `DeckCounts` and note that this is a generic structure used for all encounter-internal deck tracking (enemy decks, ore decks, and any future encounter card pools).

#### 2. `is_finished` removed — use `outcome != Undecided`
**Current vision.md describes:** Combat/encounter state with an `is_finished` field.
**Actual implementation now:** The `is_finished` field has been removed from both `CombatEncounterState` and `MiningEncounterState`. Encounter completion is determined solely by `outcome != EncounterOutcome::Undecided`. An `is_finished()` helper method on `EncounterState` provides the check.
**Suggested update:** Remove any references to `is_finished` as a field. Document that encounter completion is determined by the `EncounterOutcome` enum value.

#### 3. `encounter_card_id` is mandatory
**Current vision.md describes:** Encounter state with optional encounter card reference.
**Actual implementation now:** `encounter_card_id` is `usize` (not `Option<usize>`) on both `CombatEncounterState` and `MiningEncounterState`. Every encounter always knows which library card spawned it. The `EncounterState::encounter_card_id()` helper returns `usize` directly.
**Suggested update:** Document that `encounter_card_id` is always set and mandatory.

#### 4. `EncounterPhase::InEncounter` replaces `Combat` and `Gathering`
**Current vision.md describes:** Three phases: `NoEncounter`, `Combat`, `Scouting`.
**Actual implementation now:** `EncounterPhase::Combat` and `EncounterPhase::Gathering` have been merged into `EncounterPhase::InEncounter`. The three phases are now: `NoEncounter`, `InEncounter`, `Scouting`. The encounter type is determined by `EncounterState` variant, not by the phase.
**Suggested update:** Replace phase documentation with the three new phase names. Clarify that the encounter type (combat vs mining vs future types) is determined by the `EncounterState` enum variant, not by the phase.

#### 5. `last_durability_prevent` removed — prevent passed inline
**Current vision.md describes:** Mining turn flow where durability_prevent is stored on state.
**Actual implementation now:** `last_durability_prevent` has been removed from `MiningEncounterState`. The durability prevent value from the player's card is computed inline during `resolve_player_mining_card` and passed directly to `resolve_ore_play` as a parameter. This simplification was possible because the enemy plays immediately after the player, with no intervening state.
**Suggested update:** Update the mining turn flow description to note that prevent is computed and applied within a single action resolution, not stored between steps.

#### 6. `ore_tokens` replaces `ore_hp`/`ore_max_hp`
**Current vision.md describes:** Mining encounter with `ore_hp` and `ore_max_hp` integer fields.
**Actual implementation now:** `MiningEncounterState` uses `ore_tokens: HashMap<Token, i64>` with a `Token::persistent(TokenType::OreHealth)` entry. The old `ore_hp` and `ore_max_hp` fields have been deleted. Ore damage is applied by decrementing the OreHealth token. The ore is defeated when OreHealth ≤ 0.
**Suggested update:** Document that ore node health uses the token system (consistent with "everything is a token" principle). The serialized JSON shows `"ore_tokens": {"OreHealth": 15}`.

#### 7. Rewards use `Token` keys (not `TokenType`)
**Current vision.md describes:** Rewards as `HashMap<TokenType, i64>`.
**Actual implementation now:** `MiningDef.rewards` and `MiningEncounterState.rewards` use `HashMap<Token, i64>` — the full `Token` struct (with `token_type` and `lifecycle`) is the key, not just `TokenType`. This allows rewards to specify the lifecycle of granted tokens (e.g., persistent vs encounter-scoped).
**Suggested update:** Update reward documentation to use `Token` keys. Note this pattern should be followed by all future encounter types.

#### 8. Mining has no failure penalties
**Current vision.md describes:** Mining failure applying Exhaustion penalty.
**Actual implementation now:** `failure_penalties` has been removed from both `MiningDef` and `MiningEncounterState`. When MiningDurability reaches 0, the encounter simply ends as `PlayerLost` with no additional penalties. The rationale: MiningDurability loss IS the penalty — it's a persistent resource that decreases across encounters.
**Suggested update:** Remove references to mining failure penalties. Clarify that durability loss across encounters is the implicit penalty for mining. Future encounter types may re-introduce failure penalties if their design warrants it.

#### 9. `MiningDurability` renamed from `Durability`
**Current vision.md describes:** `TokenType::Durability` as a general-purpose durability token.
**Actual implementation now:** Renamed to `TokenType::MiningDurability` to make it discipline-specific. This supports the vision's plan for discipline-specific durability (each gathering profession has its own durability pool). Woodcutting, Herbalism, etc., would add `WoodcuttingDurability`, `HerbalismDurability`, etc.
**Suggested update:** Update token type references. Add a note that durability tokens are discipline-specific by naming convention (`{Discipline}Durability`).

#### 10. MiningDurability initialized at game start (100)
**Current vision.md describes:** Durability initialized to 15 when starting a mining encounter.
**Actual implementation now:** MiningDurability is initialized to 100 in `GameState::new()` as part of the initial token balances. It is NOT re-initialized per encounter. It persists and decreases across all mining encounters. When it reaches 0, the current encounter is lost. This models long-term profession wear — the player must eventually replenish durability through crafting/rest mechanics.
**Suggested update:** Update durability documentation to reflect game-start initialization (100) and cross-encounter persistence. Note that the high initial value (100) is a placeholder; balancing will adjust this once repair mechanics exist.

#### 11. `EncounterAbort` player action added
**Current vision.md describes:** Four player actions (NewGame, EncounterPickEncounter, EncounterPlayCard, EncounterApplyScouting).
**Actual implementation now:** A fifth player action `EncounterAbort` has been added. It allows the player to abort non-combat encounters (currently only Mining). Aborting marks the encounter as `PlayerLost`, grants no rewards, applies no penalties, and transitions to Scouting phase. Attempting to abort a combat encounter returns a 400 error. The action is recorded in the action log and handled by `replay_from_log`.
**Suggested update:** Update the player actions list to include `EncounterAbort`. Document the restriction that only non-combat encounters can be aborted (combat must be played to completion). Note: the current four-action-only claim in vision.md line 71 is now outdated.

### roadmap.md Updates

#### 1. Step 8 sub-step: docs/issues.md cleanup
Add a sub-section documenting the 10 issues resolved:
- **DeckCounts generalization**: `EnemyCardCounts` + `OreCardCounts` → `DeckCounts`
- **is_finished removal**: Use `outcome != Undecided` instead
- **Mandatory encounter_card_id**: `Option<usize>` → `usize`
- **InEncounter phase**: `Combat` + `Gathering` → `InEncounter`
- **Inline durability prevent**: `last_durability_prevent` removed from state
- **ore_tokens**: `ore_hp`/`ore_max_hp` → `HashMap<Token, i64>` with OreHealth
- **Token-keyed rewards**: `HashMap<TokenType, i64>` → `HashMap<Token, i64>`
- **No mining penalties**: `failure_penalties` removed
- **MiningDurability rename**: `Durability` → `MiningDurability`
- **Game-start durability**: Initialize at 100 in `GameState::new()`
- **EncounterAbort action**: New player action to abort non-combat encounters

#### 2. Update player actions count
The roadmap and vision reference "four player actions." This is now five: `NewGame`, `EncounterPickEncounter`, `EncounterPlayCard`, `EncounterApplyScouting`, `EncounterAbort`.

#### 3. Replay system extensibility note
The replay system (`replay_from_log`) now handles 5 action types. Each new action type must extend the replay match arm. The `EncounterAbort` pattern (calling `gs.abort_encounter()` then transitioning phase) is a good template for future encounter-ending actions.

### Contradictions Found

#### 1. vision.md line 67 says "Players cannot abandon combat once started"
This is still true for combat but needs clarification: non-combat encounters (Mining) CAN be abandoned via `EncounterAbort`. Update to: "Players cannot abandon combat encounters once started. Non-combat encounters may be aborted via EncounterAbort."

#### 2. vision.md line 71 says "Only four player actions exist"
Now five. Update the list to include `EncounterAbort`.

#### 3. vision.md line 91-94 lists three encounter phases with `Combat` variant
`EncounterPhase::Combat` has been merged into `EncounterPhase::InEncounter`. Update to: `NoEncounter`, `InEncounter`, `Scouting`.

#### 4. vision.md line 53 mentions `Durability` token type
Renamed to `MiningDurability`. Update the token type list.

#### 5. vision.md line 60 mentions `EnemyCardCounts`
Renamed to `DeckCounts`. Update the reference.

#### 6. vision.md describes ore_hp/ore_max_hp but these fields no longer exist
Ore health is now tracked via the `ore_tokens` HashMap with an OreHealth token entry.

### Copilot Instruction Suggestions

#### 1. Update player action list in copilot instructions
The instructions mention "Only four player actions exist." Update to five, adding `EncounterAbort`.

#### 2. Document the `DeckCounts` shared type
When describing enemy or ore deck card tracking, reference the shared `DeckCounts { deck, hand, discard }` struct.

#### 3. Document the Token-keyed reward pattern
Rewards use `HashMap<Token, i64>` (full Token with lifecycle), not `HashMap<TokenType, i64>`. This should be followed for all future encounter type rewards.

#### 4. Note the inline-computation pattern
When the enemy/ore plays immediately after the player (no intervening state), intermediate values (like durability_prevent) should be passed as function parameters rather than stored on state. This reduces state complexity.
