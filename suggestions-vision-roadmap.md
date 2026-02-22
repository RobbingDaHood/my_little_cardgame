
#### Changes to vision.md suggestions

14. **TokenId is now a strongly-typed enum**: The vision mentions tokens by name (Health, Insight, Foresight, etc.) but should document that token identifiers are a closed `TokenId` enum (not strings). Each variant carries lifecycle metadata via `TokenId::lifecycle()`. The current 15 variants are: Health, MaxHealth, Shield, Stamina, Dodge, Mana (combat), Insight, Renown, Refinement, Stability, Foresight, Momentum, Corruption, Exhaustion, Durability (persistent). New tokens require adding an enum variant.

15. **ScopedToEncounter lifecycle deleted**: The vision and codebase previously had a `ScopedToEncounter` token lifecycle. This has been replaced with `FixedTypeDuration { phases, duration }` parameterized by encounter phases. Vision should remove any reference to `ScopedToEncounter` and document `FixedTypeDuration` as the mechanism for phase-aware token expiry.

16. **Combat is unified into GameState**: The vision should clarify that combat state (CombatSnapshot, combat phase, last result) lives in `GameState`, not as a separate HTTP-driven system. The old `src/combat/` module now delegates to `GameState` methods (`start_combat`, `resolve_player_card`, `resolve_enemy_play`, `advance_combat_phase`). There is no longer a separate CombatState type.

17. **Dodge absorption mechanic**: The vision mentions Defence cards but should document the dodge absorption mechanic: when a player or enemy takes damage, dodge tokens are consumed first before health is reduced. This is an important combat interaction not currently described in vision.md.

18. **Resource cards are the draw engine**: The vision should explicitly state that `Resource` cards are the only way to draw additional cards. Each `CardKind::Resource` variant has a `draw_count: u32` field specifying how many random cards are drawn when the resource card is played. This is the core pacing mechanic.

19. **Enemy plays one card from each deck type per turn**: The vision says "enemy resolves its actions according to its card script" but should clarify the current simple behavior: the enemy plays one random card from each of its three deck types (attack, defence, resource) per turn. Card scripts are a future enhancement.

20. **Starting deck composition matters**: The vision should document that 50% of the starting ressource deck is drawing cards ressource cards. 

21. **AreaDeck has deck/hand/discard zones**: The vision describes area decks but should clarify that it is implicit in the LIbrary: So nto explicit in the code. The hand represents visible/pickable encounters controlled by the Foresight token. Default Foresight is 3.

22. **AbandonCombat action exists**: The vision should mention that players cannot abandon a combat after it is started. Either the player looses or the enemy is defeated. Remove all references that states otherwise. 

23. All amoutns are positive: So attacks have a positive number, even though they remove health points. This is so we can use unsigned integers as many places as possible. 

#### Changes to roadmap.md suggestions

14. **Steps 7.5 and 7.6 are now implemented**: The roadmap should mark steps 7.5 and 7.6 as complete. All playable acceptance criteria are met: unified combat system, resource-card driven draws, Foresight-controlled encounter hands, enemy random play, ~50% draw cards in starting decks, and the minimal pick→fight→scouting→pick loop works.

15. **Legacy code fully removed**: The roadmap should note that all legacy deck types (`Deck`, `DeckCard`, `CardState`, `Card`, `Token`), the old `resolve.rs` combat module, and unused helper functions have been removed. The codebase is clean of dead code.

16. **Coverage requirement met**: Step 7.5 mentioned CI failures due to <85% coverage. Coverage is now at 85.86% with comprehensive integration and unit tests. The roadmap should note this is resolved.

17. **FinishScouting is player-driven**: The roadmap should clarify that `FinishScouting` is a player action (not system-driven) that transitions from Scouting → Ready phase. The system only enters Scouting automatically when combat ends with one side at 0 HP.

18. Add Action to step 8: Ensure that all actions the player is doing is recorded in the Action log: so that given the seed and the action log then the full game state can be reproduced. Only the actions of the player need to be logged in the Action log.

#### Contradictions found and resolved

1. **Two combat systems → unified**: The codebase had both `src/combat/` (old HTTP-driven) and `library::combat` (new deterministic). These are now unified — `src/combat/` endpoints delegate to `GameState` methods. The old `resolve_card_effects` function has been deleted.

2. **String token IDs → enum**: `Combatant.active_tokens`, `CardEffect.token_id`, etc. all used `String` keys. These are now `TokenId` enum throughout. JSON serialization uses capitalized variant names ("Health", "Shield", etc.).

3. **Deck module was dead code**: `src/deck/mod.rs`, `src/deck/card.rs`, `src/deck/token.rs` contained types (`Deck`, `DeckCard`, `CardState`, `Card`, `Token`, `CardType`) that were no longer referenced anywhere. All deleted.

4. **`player_seed.rs` contained dead helpers**: `derive_subseed`, `snapshot_rng`, `restore_rng_from_snapshot` were never called. Removed.
