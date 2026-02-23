
#### Changes to vision.md suggestions

14. **TokenType is the canonical enum; Token is a struct**: The vision mentions `TokenId` in several places. `TokenId` has been renamed to `TokenType` (a closed enum: Health, MaxHealth, Shield, etc.). A `Token` is now a struct with `token_type: TokenType` and `lifecycle: TokenLifecycle`. Token maps use `Token` as the key and `i64` as the value. Vision should update all references from `TokenId` to `TokenType` and document the `Token` struct.

15. **ScopedToEncounter lifecycle deleted**: The vision and codebase previously had a `ScopedToEncounter` token lifecycle. This has been replaced with `FixedTypeDuration { phases, duration }` parameterized by encounter phases. Vision should remove any reference to `ScopedToEncounter` and document `FixedTypeDuration` as the mechanism for phase-aware token expiry.

16. **Combat is unified into GameState**: The vision should clarify that combat state (CombatSnapshot, combat phase, last result) lives in `GameState`, not as a separate HTTP-driven system. The old `src/combat/` module now delegates to `GameState` methods (`start_combat`, `resolve_player_card`, `resolve_enemy_play`, `advance_combat_phase`). There is no longer a separate CombatState type.

17. **Dodge absorption mechanic**: The vision mentions Defence cards but should document the dodge absorption mechanic: when a player or enemy takes damage, dodge tokens are consumed first before health is reduced. This is an important combat interaction not currently described in vision.md.

18. **Resource draw_count is a card effect, not a field**: The vision should state that drawing additional cards is modelled as a `DrawCards` variant of `TokenType` with a card effect, not a `draw_count: u32` field on `CardKind::Resource`. Resource cards trigger draws via their effects list, just like attack and defence cards trigger damage/shield via effects.

19. **Enemy plays one card matching the current CombatPhase**: The vision says "enemy resolves its actions according to its card script". The current implementation has the enemy play one random card from the deck matching the current `CombatPhase` (attack card during Attacking, defence during Defending, resource during Resourcing). Vision should update to reflect this per-phase behavior instead of "one card from each deck".

20. **Starting deck composition matters**: The vision should document that 50% of the starting resource deck is drawing cards resource cards.

21. **AreaDeck is implicit in the Library**: The vision describes area decks but should clarify that the AreaDeck is not a separate struct. Encounter cards live in the Library and use the same `CardCounts` (library/deck/hand/discard) as all other card types. The hand represents visible/pickable encounters controlled by the Foresight token. Default Foresight is 3. Helper methods like `encounter_hand()`, `encounter_contains()`, and `encounter_draw_to_hand()` on Library query encounter cards by their counts.

22. **Players cannot abandon combat**: The vision should remove any references to players being able to abandon combat. Once combat starts, it continues until one side is defeated (HP reaches 0). The `AbandonCombat` action has been removed.

23. **All amounts are positive**: Attacks have a positive number even though they remove health points. This allows unsigned integers to be used where possible.

24. **CombatOutcome replaces CombatResult**: The vision should document `CombatOutcome` as an enum with variants `Undecided`, `PlayerWon`, `EnemyWon` on `CombatSnapshot`. The separate `CombatResult` struct has been deleted.

25. **Encounter replaces CombatEncounter with EncounterKind**: The vision should update `CardKind::CombatEncounter` to `CardKind::Encounter { kind: EncounterKind }` where `EncounterKind` is an enum starting with `Combat { combatant_def }`. This prepares for future non-combat encounter types (Gathering, Puzzle, etc.).

26. **Player tokens live on GameState, not CombatSnapshot**: The vision should clarify that player tokens (Health, Shield, etc.) live on `GameState.token_balances` and are not part of `CombatSnapshot`. Only enemy tokens are tracked within the combat snapshot. Card effects that target the player modify `GameState` directly.

27. **Only four player actions remain**: The vision should document the complete set of player actions: `NewGame { seed: Option<u64> }`, `EncounterPickEncounter { card_id }`, `EncounterPlayCard { card_id }`, `EncounterApplyScouting`. All other previously documented actions (SetSeed, PlayCard, GrantToken, AbandonCombat, FinishScouting, ApplyScouting, DrawEncounter, ReplaceEncounter) have been removed.

28. **Auto-advance combat**: The vision should document that after a player plays a card via `EncounterPlayCard`, the system automatically resolves the enemy's play (one card matching the current phase) and advances the combat phase. There are no separate endpoints for enemy play or phase advancement.

29. **NewGame replaces SetSeed**: The vision should document `NewGame { seed: Option<u64> }` as the action that initializes a fresh game. If no seed is provided, a random one is generated. The old `/player/seed` endpoint and `SetSeed` action have been removed.

30. **Token lifecycle is dynamic per Token instance**: The vision should clarify that `TokenLifecycle` is not statically determined by `TokenType`. A `Token` consists of `(TokenType, TokenLifecycle)` and different instances of the same `TokenType` can have different lifecycles. For example, Health is typically `PersistentCounter` but could theoretically be granted with a different lifecycle.

#### Changes to roadmap.md suggestions

14. **Steps 7.5 and 7.6 are now implemented**: The roadmap should mark steps 7.5 and 7.6 as complete. All playable acceptance criteria are met: unified combat system, resource-card driven draws, Foresight-controlled encounter hands, enemy random play, ~50% draw cards in starting decks, and the minimal pick→fight→scouting→pick loop works.

15. **Legacy code fully removed**: The roadmap should note that all legacy deck types (`Deck`, `DeckCard`, `CardState`, `Card`, `Token`), the old `resolve.rs` combat module, and unused helper functions have been removed. The codebase is clean of dead code.

16. **Coverage requirement met**: Step 7.5 mentioned CI failures due to <85% coverage. Coverage is now at 85.86% with comprehensive integration and unit tests. The roadmap should note this is resolved.

17. **Post-7.6 cleanup completed**: The roadmap should add a note that the following cleanup items have been implemented after 7.6:
   - Removed 8 dead/redundant player actions (AbandonCombat, FinishScouting, ApplyScouting, DrawEncounter, ReplaceEncounter, GrantToken, PlayCard, SetSeed)
   - Consolidated combat endpoints (enemy_play and advance moved to test-only; auto-advance added)
   - Replaced SetSeed with NewGame action
   - Removed explicit AreaDeck struct in favor of Library-backed queries
   - Renamed TokenId → TokenType, created Token struct with dynamic lifecycle
   - Deleted CombatResult in favor of CombatOutcome enum
   - Moved draw_count to card effects (DrawCards TokenType)
   - Renamed CombatEncounter → Encounter with EncounterKind
   - Enemy plays one card per phase (not one from each deck)
   - Player tokens moved out of CombatSnapshot to GameState
   - Action log audited: only player actions are logged

18. **Roadmap step 8 should reference the four canonical actions**: Step 8 (expand encounter variety) should note that all gameplay flows must use the four remaining player actions. New encounter types (gathering, crafting) will need to either extend these actions or add new well-defined variants to `PlayerActions`.

19. **Action log records only player actions**: The roadmap previously stated "every grant/consume/expire/transfer must be recorded in the actions log". This should be updated: only player actions (NewGame, EncounterPickEncounter, EncounterPlayCard, EncounterApplyScouting) are recorded. Internal operations (token grants, consumes, card movements) are deterministic consequences of player actions and the seed, so they don't need logging for reproducibility.

20. **Replay from action log needs expansion**: The current `replay_from_log` function handles legacy token log entries but does not fully replay player actions (NewGame, EncounterPickEncounter, etc.). For step 8 and beyond, the replay system should be expanded to re-execute player actions against a fresh GameState initialized with the recorded seed, achieving true game state reproduction from seed + action log.

#### Contradictions found and resolved

1. **Two combat systems → unified**: The codebase had both `src/combat/` (old HTTP-driven) and `library::combat` (new deterministic). These are now unified — `src/combat/` endpoints delegate to `GameState` methods. The old `resolve_card_effects` function has been deleted.

2. **String token IDs → enum → struct**: `Combatant.active_tokens`, `CardEffect.token_id`, etc. originally used `String` keys. These were changed to `TokenId` enum and then to `TokenType` enum. Token maps now use `Token { token_type, lifecycle }` as keys with `i64` values. JSON serialization uses array-of-entries format since struct keys can't be JSON object keys.

3. **Deck module was dead code**: `src/deck/mod.rs`, `src/deck/card.rs`, `src/deck/token.rs` contained types (`Deck`, `DeckCard`, `CardState`, `Card`, `Token`, `CardType`) that were no longer referenced anywhere. All deleted.

4. **`player_seed.rs` fully removed**: The module originally contained seed management and was later reduced to dead helpers (`derive_subseed`, `snapshot_rng`, `restore_rng_from_snapshot`). The entire module and `/player/seed` endpoint have been removed; seed management is now handled by the `NewGame` action.

5. **Vision says AreaDeck has deck/hand/discard zones as a struct**: The implementation now uses Library CardCounts instead of a separate AreaDeck struct. Vision should be updated to reflect this Library-centric approach.

6. **Vision says "enemy plays one card from each deck type"**: The implementation now has the enemy play one card matching the current CombatPhase (not one from each deck). Vision needs updating.

7. **Roadmap says "every grant/consume/expire must be logged"**: The action log now only records player actions for reproducibility. Internal token operations are deterministic from player actions + seed. Roadmap needs updating.

8. **Vision references `TokenId::lifecycle()` static method**: Token lifecycle is now dynamic per Token instance, not static per TokenType. The `lifecycle()` method has been removed from the enum.

#### Areas of improvement

1. **Replay system is incomplete**: `replay_from_log` currently only handles legacy token operations (GrantToken, ConsumeToken, ExpireToken). It should be extended to replay player actions (NewGame, EncounterPickEncounter, EncounterPlayCard, EncounterApplyScouting) to achieve the vision's goal of full game state reproduction from seed + action log.

2. **Token registry entries (TokenRegistryEntry) could use cleanup**: The `TokenRegistryEntry` struct still has `lifecycle` and `cap` fields, but lifecycle is now dynamic on Token instances. Consider whether the registry entry's lifecycle is a "default lifecycle" or if it should be removed.

3. **Test coverage for non-combat encounters**: The current test suite thoroughly covers the combat encounter loop but has no tests for future encounter types. When adding gathering/crafting encounters (step 8), ensure the encounter type validation in `EncounterPickEncounter` is tested.

4. **JSON serialization of Token maps is verbose**: The array-of-entries format (`[{token: {token_type, lifecycle}, value: N}, ...]`) works but is verbose. Consider whether a more compact serialization (e.g., `"Health:PersistentCounter": 20`) would be better for API consumers.

5. **CombatSnapshot could be further simplified**: With player tokens now external, CombatSnapshot only tracks enemy state and combat metadata. Consider whether it should be renamed to reflect this (e.g., `CombatState` or `EnemyCombatState`).

