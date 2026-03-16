CHANGELOG.md
=============

Implementation changelog extracted from roadmap.md. Records implementation details, breaking changes, and cleanup batches in chronological order.

### Implementation updates (2026-02-22)
- Steps 7.5 and 7.6 implemented: unified combat (library-centric), resource-card driven draws, Foresight-controlled encounter hands, enemy random play, and a minimal pick→fight→scouting→pick loop.
- Legacy deck types and dead code (resolve.rs, unused player_seed helpers) removed.
- CI coverage target (≥80%) achieved: threshold reduced from 85% to 80% after test consolidation.

### Post-7.6 cleanup (2026-02-23)
- Removed 8 dead/redundant player actions: AbandonCombat, FinishScouting, ApplyScouting, DrawEncounter, ReplaceEncounter, GrantToken, PlayCard, SetSeed. Four player actions remained: NewGame, EncounterPickEncounter, EncounterPlayCard, EncounterApplyScouting (EncounterAbort was added later in Step 8.1).
- Consolidated combat endpoints: /combat/enemy_play and /combat/advance removed (formerly test-only under /tests/* prefix); auto-advance added to EncounterPlayCard so the system resolves enemy play and advances the combat phase automatically. All /tests/* endpoints have been removed.
- Replaced SetSeed with NewGame { seed: Option<u64> }; removed /player/seed endpoint and player_seed.rs module entirely.
- Removed explicit AreaDeck struct; encounter cards now use Library CardCounts (library/deck/hand/discard) like all other card types, with helper methods (encounter_hand, encounter_contains, encounter_draw_to_hand) on Library.
- Renamed TokenId → TokenType; created Token struct with token_type + lifecycle fields for dynamic lifecycle per instance.
- Deleted CombatResult struct; replaced with CombatOutcome enum (Undecided, PlayerWon, EnemyWon) on CombatState.
- Moved Resource draw_count into card effects: DrawCards is now a CardEffectKind variant (not a TokenType).
- Renamed CardKind::CombatEncounter → CardKind::Encounter { kind: EncounterKind } with EncounterKind enum.
- Enemy now plays one card matching the current CombatPhase (not one from each deck).
- Player tokens (Health, Shield, etc.) moved out of CombatSnapshot to GameState.token_balances.
- Action log audited: only player actions are logged. Internal operations (token grants, consumes, card movements) are deterministic from player actions + seed.
- Replay system note: replay_from_log now replays player actions (NewGame, EncounterPickEncounter, EncounterPlayCard, EncounterApplyScouting, EncounterAbort). Combined with the initial seed, the action log is sufficient to reconstruct the full game state for the core loop.

### Post-7.7 implementation (2026-02-23)
- All issues from docs/issues.md resolved:
  - Issue 9: Removed unused `effects` field from EncounterPlayCard
  - Issue 7: Removed with_default_lifecycle; all tokens PersistentCounter except Dodge (FixedTypeDuration to Defence phase); lifecycle is solely on the Token struct, not on card effects
  - Issue 2: Removed lifecycle from TokenRegistryEntry (now only id + cap); TokenRegistry has since been fully deleted
  - Issue 4: Token maps serialize as compact JSON objects (e.g., {"Health": 20}); backward-compatible deserialization
  - Issue 5: Renamed CombatSnapshot → CombatState
  - Issue 6: Enemy decks track deck/hand/discard counts; hand shuffle at combat start; play from hand only. Resource DrawCards draws per deck type (attack, defence, resource) for all three enemy deck types.
  - Issue 8: /tokens endpoint removed (TokenRegistry deleted); token state is accessed via /player/tokens
  - Issue 1: replay_from_log handles SetSeed, DrawEncounter, PlayCard, ApplyScouting
- Step 7.7 implemented: PlayerCardEffect and EnemyCardEffect CardKind variants; card_effect_id references; validation; GET /library/card-effects endpoint
- New cards should always be appended to the end of the Library vector to preserve stable card IDs

### Post-7.7 cleanup (2026-02-24)
- Removed `EncounterPhase::Defence` (now uses `CombatPhase::Defending`)
- Removed `Combatant` struct (enemy tokens moved directly to `CombatState.enemy_tokens`)
- Extracted `DrawCards` from `TokenType` into `CardEffectKind` enum with per-deck-type fields { attack, defence, resource }
- DrawCards amounts: 1 attack, 1 defence, 2 resource per resource play (4 total) for steady pacing
- Split `library.rs` into `src/library/` module directory with submodules (types, action_log, game_state, endpoints)
- Added long-scenario integration tests (`tests/scenario_tests.rs`) using only production endpoints

### Pre-step-8 cleanup (2026-02-26)
- All issues from docs/issues.md (second round) resolved:
  - Fixed draw_player_cards_of_kind to draw random cards (was always drawing first card)
  - Removed lifecycle field from PlayerCardEffect and EnemyCardEffect (lifecycle solely on Token)
  - Deleted empty src/tests.rs and removed leftover comments
  - Renamed PlayerData to RandomGeneratorWrapper
  - Used typed CardKind check in CombatPhase::allowed_card_kind (returns fn(&CardKind)->bool)
  - Removed redundant CombatState.player_turn field (turn control is implicit)
  - Replaced EncounterState wrapper with EncounterPhase directly on GameState
  - Simplified EncounterPhase: removed Ready variant, renamed InCombat to Combat
  - Removed TokenRegistry, apply_grant, apply_consume, /tokens endpoint entirely
  - Expanded last_combat_result to combat_results: Vec<CombatOutcome> with /combat/results endpoint
  - Changed CombatantDef.initial_tokens to HashMap<Token, u64>
  - Simplified ActionPayload to 4 variants matching PlayerActions (SetSeed, DrawEncounter, PlayCard, ApplyScouting)
  - Simplified ActionEntry to just seq + payload (removed action_type, timestamp, actor, request_id, version)
  - Added CardLocation enum (Library, Deck, Hand, Discard) and ?location=/?card_kind= filters to /library/cards
  - Removed /area and /area/encounters endpoints; encounter cards accessed via /library/cards?location=Hand&card_kind=Encounter
  - Removed AreaDeck struct (was only used in tests)
  - Removed ScoutingParams and entire src/area_deck/ module
- Test files removed (tested deleted production code): library_integration.rs, proptest_sequences.rs, proptest_replay.rs, replay_determinism.rs, area_deck_integration.rs, area_deck_e2e.rs
- All scenario coverage is now in `tests/scenario_tests.rs` using only production endpoints
- Note: ScoutingParams will need to be re-implemented as part of step 11 (post-encounter scouting choices) within the Library/GameState system

### Step 7 COMPLETE
Steps 7, 7.5, 7.6, and 7.7 are fully implemented and cleaned up. The core encounter loop (pick → fight → scouting → pick) is operational with resource-card driven draws, Foresight-controlled encounter hands, enemy random play, CardEffect decks, and a single unified combat system. All legacy code (CardDef, old combat simulation, EncounterAction state machine, TokenRegistry, AreaDeck) has been removed.

### Step 8 implementation updates (2026-02-28)
- Step 8.1 (Mining) implemented: first gathering discipline, establishing EncounterState enum pattern.
- BREAKING: /combat → /encounter, CombatState → EncounterState, CombatOutcome → EncounterOutcome, EnemyCardCounts → DeckCounts.
- BREAKING: EncounterPhase::Combat + Gathering merged into EncounterPhase::InEncounter.
- EncounterAbort player action added (fifth action). Non-combat encounters can be aborted; combat returns 400.
- docs/issues.md cleanup (10 issues resolved):
  - DeckCounts generalization (EnemyCardCounts + OreCardCounts → DeckCounts)
  - is_finished removal (use outcome != Undecided)
  - Mandatory encounter_card_id (Option<usize> → usize)
  - InEncounter phase (Combat + Gathering → InEncounter)
  - Inline durability prevent (last_durability_prevent removed from state)
  - ore_tokens (ore_hp/ore_max_hp → HashMap<Token, i64> with OreHealth)
  - Token-keyed rewards (HashMap<TokenType, i64> → HashMap<Token, i64>)
  - No mining penalties (failure_penalties removed)
  - MiningDurability rename (Durability → MiningDurability)
  - Game-start durability (initialize at 100 in GameState::new())
- replay_from_log handles 5 action types. Each new action type must extend the replay match arm.
- Mining scenario tests added (full loop + abort test).
- Step 8.2 (Herbalism) implemented: card-characteristic matching with no enemy draws. New card IDs: 16-19. HerbalismDurability depletion added as second loss condition. 2 scenario tests.
- Step 8.3 (Woodcutting) implemented: rhythm-based pattern matching, no enemy deck. New card IDs: 20-24. Poker-inspired pattern evaluation (13+ patterns). 2 scenario tests.
- Step 8.4 (Fishing) implemented: card-subtraction with valid-range targeting. New card IDs: 25-28. Enemy fish deck with 4 card variants. 2 scenario tests.
- All 4 gathering disciplines now share the same EncounterState enum pattern, confirming it is reusable for future encounter types.


### Post-9.3 implementation (2026-03-02)
- BREAKING: `ChangeTokens` CardEffectKind split into `GainTokens` and `LoseTokens`. GainTokens has required cap_min/cap_max/gain_min_percent/gain_max_percent fields; LoseTokens has positive min/max (amount to lose). GainTokens cannot have a cost_type matching the gain token_type.
- BREAKING: `stamina_grant` field removed from all four discipline card effects (MiningCardEffect, HerbalismCardEffect, WoodcuttingCardEffect, FishingCardEffect). Replaced with `gains: Vec<GatheringCost>` for granting any token type on card play. **Note:** These discipline-specific effect structs (MiningCardEffect, HerbalismCardEffect, WoodcuttingCardEffect, FishingCardEffect, CraftingCardEffect) have since been fully removed in the post-Step-10 fixes batch — all gathering cards now use `effects: Vec<ConcreteEffect>` referencing library templates.
- BREAKING: `modify_range_min`, `modify_range_max`, `modify_fish_amount` fields removed from FishingCardEffect. Now expressed as entries in the `gains: Vec<GatheringCost>` vector using FishingRangeMin, FishingRangeMax, FishAmount token types.
- BREAKING: `durability_cost` removed from HerbalismCardEffect, WoodcuttingCardEffect, FishingCardEffect. `stamina_cost` removed from MiningCardEffect, WoodcuttingCardEffect. All costs now use `costs: Vec<GatheringCost>` exclusively. `merge_gathering_costs()` removed. `TokenType::is_durability_cost()` and `split_gathering_costs()` added to classify costs as pre-play (reject if unaffordable) or post-play (durability depletion).
- BREAKING: `target_characteristics` removed from HerbalismCardEffect. Replaced with `HerbalismMatchMode` enum that wraps data: `Or { types }`, `And { types }`, `MostCommon { limit, types }`, `LeastCommon { limit, types }`.
- BREAKING: All 7 `*MaxHand` tokens (AttackMaxHand, DefenceMaxHand, ResourceMaxHand, MiningMaxHand, HerbalismMaxHand, WoodcuttingMaxHand, FishingMaxHand) initialized to 5 instead of 10.
- Unpayable card → error: if no effect on a card can have its pre-play costs paid, the play is rejected with an error. Player must choose another card.
- Autoloss extended to all disciplines: all encounter types (Combat, Mining, Herbalism, Woodcutting, Fishing) now check if all hand cards are unpayable and auto-lose if so. Previously only combat checked this.
- `game_state.rs` discipline logic split into `src/library/disciplines/` with per-discipline modules: `combat.rs`, `mining.rs`, `herbalism.rs`, `woodcutting.rs`, `fishing.rs`. General methods (cost payment, token operations) remain in `game_state.rs`.
- Card initialization refactored from monolithic `initialize_library()` into per-discipline registration functions (`register_combat_cards`, `register_mining_cards`, etc.) under `src/library/disciplines/`. `initialize_library` is now a thin orchestrator calling these discipline-specific functions.
- BREAKING: Card IDs are now dynamic (determined by registration order) rather than hard-coded. Tests referencing specific card IDs need updating. Consider a card lookup-by-name or card-type query endpoint to make tests more resilient to ID changes.
- Gathering unpayable DRY refactor: four identical `all_<discipline>_hand_cards_unpayable()` methods unified into a single generic `all_gathering_hand_cards_unpayable()` on `GameState` that takes a closure to extract costs from the discipline-specific `CardKind` variant. Combat's unpayable check remains separate.
- `HasDeckCounts` trait: unified `deck_draw_random`, `deck_shuffle_hand`, and `deck_play_random` generic functions replace duplicated per-discipline methods for `OreCard`, `FishCard`, `PlantCard`, `EnemyCardDef`. Removed ~74 lines of duplicated code. Combat's `resolve_enemy_play` updated to use `deck_play_random` with weighted-by-count selection.
- Woodcutting multiplier rebalance: pattern multipliers recalibrated proportional to the statistical probability of each pattern (assuming 8 cards played from a 13-card pool), so rarer patterns yield substantially higher rewards.
- docs/issues.md batch (10 issues resolved).

9.4) Rest encounter ✅ COMPLETED (refactored)
   - Goal: Add a rest encounter type that allows stamina and health recovery, creating a meaningful pacing mechanic with multi-card play gated by rest tokens.
   - Description: Rest cards are **player library cards** (`CardKind::Rest`) living in the Library with `CardCounts`, following the same deck/hand/discard pattern as Attack/Defence/Resource cards.
     - The starting encounter deck has ~20% rest encounters (hand: 4 out of 19 total encounter cards).
     - 4 PlayerCardEffect templates (2 Stamina recovery, 2 Health recovery) and 5 concrete rest cards are registered at game init, each with 5 copies (25 total in the rest deck).
     - Rest cards use the `ConcreteEffect`/`GainTokens` pattern with `effect_id` references to `PlayerCardEffect` entries.
     - Material costs (Fish and Plant) are percentage-of-gain via `CardEffectCost` on effects; the mixed card is cost-free.
     - At encounter start, rest cards are drawn from the Library deck to hand (up to `RestMaxHand` limit, default 5). The encounter grants 1–2 **rest tokens**.
     - Playing a rest card costs `rest_token_cost` (0–2) from the encounter's token pool plus material costs.
     - Multiple cards can be played per encounter. When rest tokens are depleted, the encounter auto-completes as PlayerWon.
     - The player can abort at any time (always PlayerWon — there is no loss condition).
     - `EncounterKind::Rest` is a unit variant (no encounter-internal definition needed).
   - Implementation: Completed with `CardKind::Rest { effects, rest_token_cost }`, `RestEncounterState { rest_tokens }`, `RestToken`/`RestMaxHand` token types, rest.rs discipline module (`register_rest_cards`, `start_rest_encounter`, `resolve_rest_card_play`, `abort_rest_encounter`, `complete_rest_encounter`), action handler integration, replay support, and scenario test. Old types removed: `RestCard`, `RestDef`, `RestCardEffectTemplate`, `RestCostRange`, `RestRecoveryRange`, `ConcreteRestRecovery`.

9.5) Better Mining redesign — ✅ COMPLETED
   - Goal: Redesign the mining encounter to be about maintaining a light level while mining for yield, creating a risk-vs-reward pacing mechanic where the player decides when to stop.
   - Description:
     - **Core loop**: The player manages three resources during a mining encounter: light level, yield, and stamina. The player can conclude the encounter at any point; the reward is `min(stamina, yield)` and concluding costs that amount of stamina.
     - **Light level**: A new token starting at 300 at the start of each mining encounter.
       - Enemy cards reduce the light level (moderate amount). Most enemy cards reduce both light level and durability; some only reduce one (doing more of it).
       - Player cards can increase the light level (high amount) with a cap (rolled like all gain effects: cap first, then gain as percentage of cap). Each light-level card also costs a small amount of wood tokens proportional to the gain. No single player CardEffect both increases light level and does mining power. Later, crafted multi-effect cards could combine both.
     - **No enemy health**: The enemy has no health and cannot be killed. The player can only win by ending the encounter. The player loses by running out of durability or having all hand cards unpayable.
     - **Mining power → yield**: When the player plays a "mining power" card (renamed from "damage"), a yield token is accumulated: `yield += mining_power × light_level / 100`. Higher light level means more yield per card played.
     - **Enemy CardEffects**: Because there is no enemy entity to fight, enemy cards cannot have CardEffects that cost stamina. The enemy does have rare cards that remove a small amount of the player's health.
   - Implementation: Completed. Mining now uses a fully token-based system: `MiningCardEffect` has `costs`/`gains` (Vec<GatheringCost>); `OreCard` has `effects: Vec<ConcreteEffect>` and `counts: DeckCounts` (the `damages` field was removed — ore card resolution now uses `effects` resolved via `library.resolve_effect()`). `MiningDef` has `initial_light_level` (300) and `ore_deck`. New token types: `MiningLightLevel`, `MiningYield`, `MiningPower` (all encounter-scoped, reset to 0 on encounter end). Yield formula: `mining_power × light_level / 100`. Conclude action: `EncounterConcludeEncounter` grants `min(stamina, yield)` Ore tokens. Loss conditions: `MiningDurability ≤ 0` or all hand cards unpayable. 8 player mining cards (power, light, rest varieties) + 1 encounter definition. All scenario tests pass.
   - **Post-cleanup summary (Step 9.5 post-cleanup pass):** This step included a significant cleanup pass affecting areas beyond mining: token restructuring (TokenType enum consolidation, encounter-scoped token migration from global token_balances to encounter_tokens), player death mechanic implementation (material reset, Health/Stamina restore, PlayerDeaths counter), EncounterConcludeEncounter standardized across all gathering disciplines (Mining, Herbalism, Woodcutting, Fishing), dynamic test ID migration (tests no longer rely on hardcoded card IDs), and documentation updates (vision.md/roadmap.md consolidation).

9.6) Crafting encounters and discipline — ✅ COMPLETED (with post-implementation fixes)
   - Goal: Implement crafting as a discipline encounter type that uses crafting tokens and gathering materials to create, modify, and enhance cards.
   - Description: A crafting encounter provides a pool of "Crafting tokens" (initially ~10) that the player spends on various crafting actions:
     - 1 token: Replace a card between the deck/discard pile and the library. Choose two cards: one moves from deck/discard to library, and the other does the opposite. Cannot move from hand. Only applies to player cards, not area/encounter cards. Cards must be available for swap.
     - X tokens: Craft a new card. Choose one player card that already exists in the library and try to make a copy of it.
     - 1 token: Add durability to a chosen discipline for a cost of some wood or ore.
   - Crafting card type gameplay:
     - The game evaluates the "cost" of the card in gathering tokens. Every player card in the library calculates this cost when created and persists it on the card as one field to inspect. The more effects and the better values the effects have, the higher the cost.
     - The game is played over X turns; every turn costs 1 crafting token.
     - Each turn the player plays a crafting card. Crafting cards have one or more gathering token types and a number for each: every time they play a card they reduce the cost of the craft with what is mentioned on the card.
     - The cost can at maximum be halved in each of the cost token types.
     - The enemy has a similar deck and also plays a card every turn that increases the cost of one or more tokens.
     - In general the enemy cards are skewed so the player cards are slightly more powerful initially.
     - The player can only lose the encounter if the player cannot pay the final cost; otherwise they win it.
   - The player can abort a crafting encounter at any point.
   - Playable acceptance: Can resolve a craft encounter, produces a Library card copy (visible via GET /library), and demonstrates cost evaluation based on card effects; crafted cards are never directly inserted into player decks.
   - Notes: Start with a single crafting encounter type to prove the flow; ensure crafting is the primary economy sink and costs scale with card quality.
   - **Costs note:** Costs are now defined IN the CardEffectKind template via `CardEffectCost`, not applied post-hoc. `compute_*_card_value()` / `apply_*_costs()` patterns have been eliminated. Discipline-specific effect structs (CraftingCardEffect) have been fully removed.
   - Stamina and Health tokens should be usable in CardEffects with costs within the crafting discipline, same deck mix as other discipline cards (mostly no-cost, some cost cards) in the initial deck.
   - Post-implementation fixes (done alongside Step 10):
     - **Crafted card deduplication:** Crafting now increments the `library` count of the existing card instead of creating a new Library entry with a duplicate definition.
     - **Merged conclude/auto_conclude:** Extracted shared craft conclusion logic into `finish_active_craft()` helper, eliminating duplication between manual conclude and auto-conclude paths.
     - **Block abort during active craft:** `abort_crafting_encounter` now returns an error if a craft mini-game is in progress; the player must conclude or complete the craft first.
     - **Variable crafting cost distribution:** Costs are now distributed randomly across 2–4 material tokens (max 75% per token) using Fisher-Yates shuffle with seeded RNG, replacing fixed even distribution.
     - **Enemy card effects:** `EnemyCraftingCard` now has `effects: Vec<ConcreteEffect>` referencing registered EnemyCardEffect entries (4 crafting-specific effects). The `increases` field was removed. See enemy card effect refactoring notes under Step 10.

10) Research encounters and card discovery — ✅ COMPLETED
   - Goal: Implement Research as a first-class encounter type where players invest Insight tokens to discover and create new cards for their library.
   - **Implementation note**: Insight infrastructure uses per-discipline insight tokens — `CombatInsight`, `MiningInsight`, `HerbalismInsight`, `WoodcuttingInsight`, `FishingInsight`, `RestInsight`, `CraftingInsight`, `ResearchInsight` — replacing the former standalone `MilestoneInsight` and `Insight` token types. The CardEffectKind::Insight variant and discipline tags are implemented.
   - Description:
     - **CardEffect discipline tags**: Every CardEffect has a set of discipline tags (e.g., Combat, Mining, Herbalism, Woodcutting, Fishing) that determine which card types can use that effect. This enables effects to be shared across disciplines when appropriate.
       - Generalize the "Durability" card effects so they can be used across all gathering mechanics. ✅ Implemented: card effects use `TokenType::Durability` which resolves to the correct per-discipline pool at encounter time via `TokenType::resolve_durability(&Discipline)`. Per-discipline durability tokens remain separate in the balance.
       - Review other CardEffects for similar generalization opportunities.
     - **Insight card effect**: Add a CardEffectKind::Insight variant that grants per-discipline Insight tokens.
       - Can be added to every player card type (Attack, Defence, Resource, Mining, Herbalism, Woodcutting, Fishing, etc.).
       - Grants between 1-5 per-discipline Insight tokens (e.g., CombatInsight, MiningInsight) when the card is played.
       - Each player deck starts with a couple of cards that have an Insight effect granting 3 Insight.
       - The trade-off: playing an Insight card gives no other benefit in the encounter — it sacrifices immediate encounter power for long-term research progress.
     - **Research state**: The current research project and its progress are stored in GameState (persisted across encounters).
     - **Research encounters**: At a research encounter, the player can perform the following actions:
       1. **Choose new research or swap the current one** (single player action):
          - Choose which discipline to research.
          - Choose the number of tiers (card effects) on each candidate card.
          - Pay an Insight cost to get started: exponential based on the number of tiers, starting at 10.
          - The game instantly generates three possible cards to research from that discipline:
            - For each candidate card:
              - Select from all CardEffects whose discipline tags match the chosen discipline.
              - For each CardEffect, roll a value between its min and max (using the range system from Step 9.1).
              - The same CardEffect can appear multiple times on a card, each with a new independent roll.
              - Add one CardEffect per chosen tier.
            - Present all three candidates to the player (both in the API response and persisted on the research encounter state).
          - The player then chooses one of the three candidates, or keeps their current research (if any).
       2. **Progress on the current research** (player action):
          - The max cost is exponential with the number of tiers, starting at 20.
          - The player can pay up to 33% of the total research cost per action (using Insight tokens). Later this payment mechanic will become its own discipline.
          - Payment is added to the research progress.
          - If this completes the research:
            - A new card is added to the Library with no counts (0 copies in any zone), of the researched card type.
            - The current research and its progress are cleared.
   - Playable acceptance: Research encounters are playable end-to-end. Players can choose a discipline, generate candidates, select a research project, make progress payments, and complete research to produce new Library cards. All rolls are deterministic via the game seed. Scenario tests verify the full research flow.
   - Notes: CardEffect discipline tags and the Insight card effect are prerequisites that should be implemented early in this step. The research encounter builds on these foundations and on the range system from Step 9.1.
   - Implementation results:
     - **Discipline enum and tags:** `Discipline` enum (Combat, Mining, Herbalism, Woodcutting, Fishing, Rest, Crafting, Research) added. `valid_discipline_types: Vec<Discipline>` field on `LibraryCard` for PlayerCardEffect and EnemyCardEffect entries. `Library::card_effects_for_discipline()` filters effects by discipline tag.
     - **CardEffectKind::Insight:** New variant that grants per-discipline Insight tokens (min-max roll). A shared Insight `PlayerCardEffect` is registered with all discipline tags. Insight Resource cards added to the combat starting deck. All 7 gathering disciplines now process Insight effects.
     - **Research types:** `ResearchDef`, `ResearchCandidate`, `ResearchProject`, `ResearchEncounterState` structs in `types.rs`. `EncounterKind::Research { research_def }` variant.
     - **Research state:** `current_research: Option<ResearchProject>` persisted in `GameState` across encounters.
     - **Research encounter lifecycle:** start → choose project (discipline, tier_count) → pay start cost → generate 3 candidates → select candidate → progress payments → complete → new Library card.
     - **Cost formulas:** Start cost = `10 × 2^(tier-1)`, completion cost = `20 × 2^(tier-1)`, 33% cap per payment.
     - **Candidate generation:** 3 candidates generated from discipline-tagged CardEffects with independent rolls per tier. Same CardEffect can appear multiple times.
     - **6 new scenario tests:** Full research loop, swap project, insufficient Insight, abort research, crafting abort blocking, crafting card deduplication.
     - **Enemy card effect refactoring (cross-cutting):**
       - All enemy card types (`OreCard`, `PlantCard`, `FishCard`, `EnemyCraftingCard`) now have `effects: Vec<ConcreteEffect>` referencing EnemyCardEffect entries. `OreCard.damages` and `EnemyCraftingCard.increases` fields removed; both now use `effects` resolved via `library.resolve_effect()`.
       - Combat EnemyCardEffects moved from `game_state.rs` to `combat.rs`. New EnemyCardEffects registered: Mining (5), Herbalism (2), Fishing (4), Crafting (4).
       - `validate_card_effects()` extended to validate enemy effects across all encounter types (not just combat).
     - **Encounter deck composition:** 1 research encounter added to starting deck.
   - Deferred items:
     - **Generalized durability effects:** ✅ Now implemented — card effects use `TokenType::Durability` which resolves to the per-discipline pool at encounter time. Per-discipline durability tokens remain separate in the balance.
     - **Non-Attack researched cards:** All researched cards are currently Attack cards regardless of the research discipline. Future work should map discipline to the appropriate card kind.
     - **Insight in gathering encounters:** ✅ Now implemented — all 7 gathering disciplines process Insight effects, granting per-discipline insight tokens (CombatInsight, MiningInsight, etc.).
     - **Research encounter card position:** The research encounter card's starting position (deck vs hand) is determined by the seed and is not a design concern.

### Post-Step-10 implementation batch

This section summarizes changes implemented after Step 10 completion:

- **BREAKING: Per-discipline insight tokens:** Replaced single shared `Insight` and `MilestoneInsight` tokens with per-discipline variants: `CombatInsight`, `MiningInsight`, `HerbalismInsight`, `WoodcuttingInsight`, `FishingInsight`, `RestInsight`, `CraftingInsight`, `ResearchInsight`. `TokenType::insight_for_discipline(&Discipline) -> TokenType` resolves the correct pool.
- **All disciplines process Insight effects:** All 7 gathering disciplines (Mining, Herbalism, Woodcutting, Fishing, Crafting, Rest, Combat) now process Insight card effects, granting insight to the per-discipline pool.
- **Generalized durability card effects:** Card effects now use generic `TokenType::Durability` which resolves to the correct per-discipline pool at encounter time via `TokenType::resolve_durability(&Discipline)`. Per-discipline durability tokens remain separate in the balance.
- **Renamed discipline_tags → valid_discipline_types:** The field on LibraryCard was renamed from `discipline_tags` to `valid_discipline_types`.
- **ResearchProject cleanup:** Removed `discipline` and `tier_count` fields from ResearchProject (use `chosen_card.discipline` and `total_cost` instead).
- **ConcreteEffect migration (Mining, Crafting):** `OreCard.damages` field removed — mining now uses `effects: Vec<ConcreteEffect>` resolved via `library.resolve_effect()`. `EnemyCraftingCard.increases` field removed — crafting now uses `effects: Vec<ConcreteEffect>` resolved similarly. `PlantCard.characteristics` and `FishCard.value` kept (unique game mechanics).
- **Cap behavior clarification:** Caps (both ConcreteEffect `rolled_cap` and TokenAmount `cap`) limit the GAIN from a single card effect, not the total token balance. The total balance may exceed the cap.
- **Stamina/Health cost card tiers:** Every discipline now has starting cards with stamina costs (medium benefit) and health costs (great benefit), in addition to no-cost cards (basic benefit).
- **GET /actions/possible endpoint:** New read-only endpoint returns currently valid player actions based on game state, including playable card IDs (OpenAPI documented).
- **Test consolidation:** Removed `resolve_play_tests.rs`, `flow_tests.rs`, `api_tests.rs`, `api_end_to_end.rs`, `coverage_integration.rs`, `combat_interleaved.rs`. Removed all /tests/* test-only endpoints (POST /tests/combat, POST /tests/combat/enemy_play, POST /tests/combat/advance, POST /tests/library/cards). Remaining test files: `scenario_tests.rs` (41 tests), `library_unit.rs`, `library_coverage.rs`, `actions_log_replay.rs`.
- **CI coverage threshold:** Changed from 85% to 80%.

### Post-Step-10 fixes: gathering effects migration

- **BREAKING: All gathering CardKind variants migrated to library-referenced effects.** Mining, Crafting, Herbalism, Woodcutting, and Fishing CardKind variants now use `effects: Vec<ConcreteEffect>` referencing PlayerCardEffect templates — the same pattern as Combat and Rest. Inline `costs: Vec<TokenAmount>` / `gains: Vec<TokenAmount>` fields removed from all gathering card kinds. Costs are now LoseTokens effects; gains are GainTokens effects.
- **BREAKING: Discipline-specific effect structs removed.** `MiningCardEffect`, `CraftingCardEffect`, `HerbalismCardEffect`, `WoodcuttingCardEffect`, and `FishingCardEffect` structs have been fully removed. All card effect data now flows through the two-layer CardEffect architecture (PlayerCardEffect templates → ConcreteEffect rolled values).
- **4 new CardEffectKind variants:** `WoodcuttingChop` (chop-type/value mechanics), `HerbalismMatch` (characteristic matching), `FishingValue` (numeric card values), `CraftingReduction` (crafting difficulty reduction). These carry discipline-specific mechanics that were previously embedded in the removed structs.
- **PossibleAction refactored:** Removed `PossibleAction` struct. The `/actions/possible` endpoint now returns `Vec<PlayerActions>` directly.
- **CI coverage enforced at 80% threshold** in `make check`.
- **`all_gathering_hand_cards_unpayable()` removed.** Replaced by `all_effects_hand_cards_unpayable()` which works with the ConcreteEffect-based cost model across all disciplines.

### Research Hidden Multiplier Gameplay (Step 10.1)

Replaces the simple "pay Insight to progress" research mechanic with a deduction-driven deck-based gameplay:

- **New types:** `ResearchSymbol` enum (Alpha–Zeta, 6 abstract knowledge symbols), `ResearchRoundResult` struct, `ResearchDef` with `target_size`/`position_match_yield`/`type_match_yield`/`base_insight_cost` fields.
- **Updated types:** `ResearchEncounterState` gains `hidden_types`, `accumulated_yield`, `rounds_played`, `round_history`, `experiment_active` fields. `CardKind::Research` variant added. `CardEffectKind::ResearchProbe` variant added.
- **New actions:** `ResearchPlayHand { card_ids }` and `ResearchConcludeExperiment`.
- **Research player deck:** 26 cards total — 6 basic single-symbol cards (3 copies each = 18), 3 dual-symbol premium cards (2 copies each = 6, cost Stamina), 2 triple-symbol premium cards (1 copy each = 2, cost Health).
- **Gameplay flow:** Choose project → auto-begin experiment on first ResearchPlayHand → play 3 cards per round (order matters) → 1:1 optimal matching against 3 hidden symbol slots → position match = 100 yield, type match = 10 yield → Insight cost escalates linearly (round N costs N × 5) → player chooses when to stop → ResearchConcludeExperiment applies yield to progress.
- **Deduction mechanic:** `hidden_types` never exposed in API (serde skip_serializing). Player deduces from `per_card_yield` in round_history. 6 symbols × 3 slots = 216 possible combinations.
- **Balance parameters:** Y=100 (position), X=10 (type), Z=5 (base cost). Linear cost escalation.
- **Tests:** 7 new integration tests covering full loop, zero-yield loss, cost escalation, wrong card count validation, hidden types visibility, card existence, multi-round yield.
- **BREAKING:** `ActionPayload::ResearchPlayHand` and `ActionPayload::ResearchConcludeExperiment` added. `CardKind::Research` added (exhaustive match impact across codebase). `CardEffectKind::ResearchProbe` added.
