roadmap.md
===========

Overview
--------
This roadmap turns the high-level ideas in docs/vision.md into a sequence of concrete, incremental, and playable features that faithfully implement the vision principles. Every step emphasizes the vision constraints: "Everything is a deck or a token", a single initial RNG seed for full reproducibility, a canonical Library that owns card definitions, and explicit token lifecycle and actions logging. Each milestone produces a minimal playable loop (API or CLI) so the project can be iteratively tested, balanced, and extended.

Alignment requirements (inherited from vision.md)
------------------------------------------------
- Everything-as-deck/token: Core library types and APIs must model cards, encounters, recipes, merchants and materials as decks and individual items as tokens.
- Single-seed reproducibility: New games are created with a single explicit seed; all random operations (shuffles, rolls, selection) are derived from that seed and are replayable.
- Canonical Library semantics: The Library is the authoritative catalog of card definitions. Crafted card copies are created in the Library and never directly injected into player decks; players add Library cards into decks via deck-management flows subject to deck-type constraints.
- Token lifecycle & actions log: The action log records only player actions (not internal token operations). Combined with the initial seed, the player action log is sufficient to reproduce all game state including token lifecycles. Token definitions live in the `TokenType` enum and `Token` struct; lifecycle is declared on the Token directly (not in a separate registry).
- Single mutator endpoint: All state mutations must be performed via a single POST /action endpoint (the "action" endpoint) which accepts structured action payloads; other endpoints are read-only. The current player actions are: NewGame, EncounterPickEncounter, EncounterPlayCard, EncounterApplyScouting, EncounterAbort.
- All gameplay state mutations must be performed via POST /action. Testing and debugging endpoints under /tests/* are exceptions and should be documented as temporary testing utilities.

Roadmap steps
--------------

### Implementation updates (2026-02-22)
- Steps 7.5 and 7.6 implemented: unified combat (library-centric), resource-card driven draws, Foresight-controlled encounter hands, enemy random play, and a minimal pick→fight→scouting→pick loop.
- Legacy deck types and dead code (resolve.rs, unused player_seed helpers) removed.
- CI coverage target (≥85%) achieved: 85.86% after adding integration and unit tests.

### Post-7.6 cleanup (2026-02-23)
- Removed 8 dead/redundant player actions: AbandonCombat, FinishScouting, ApplyScouting, DrawEncounter, ReplaceEncounter, GrantToken, PlayCard, SetSeed. Four player actions remained: NewGame, EncounterPickEncounter, EncounterPlayCard, EncounterApplyScouting (EncounterAbort was added later in Step 8.1).
- Consolidated combat endpoints: /combat/enemy_play and /combat/advance moved to test-only (/tests/* prefix); auto-advance added to EncounterPlayCard so the system resolves enemy play and advances the combat phase automatically.
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

Roadmap steps
--------------

1) Implement core Library types: unify decks, hands, tokens, and enforce vision constraints
   - Goal: Create a single library crate that is the authoritative implementation of decks, tokens, Library semantics, and the canonical token definitions.
   - Description: Extract Deck, Hand, Zone, Token, Library and ActionLog types; implement token types with lifecycle metadata and a compact actions log API to record player actions.
   - Playable acceptance: Unit tests and property tests for deck/token invariants; an API endpoint GET /library/cards returns canonical card entries; actions log records player actions.
   - Notes: Make the library the only place that mutates authoritative game state; surface a small, well-documented API and enforce "everything is deck/token" at type level.

2) Implement global seeded RNG and deterministic execution primitives
   - Goal: Add a single-game RNG and a deterministic scheduler that all systems use (deck shuffles, encounter generation, CardEffect rolls, combat decisions where non-deterministic choices exist).
   - Description: Provide RNG seeding at game/session creation, utility methods to derive deterministic sub-seeds, and deterministic replay helpers (serialize/deserialize seeds and RNG state). Integrate RNG into the ActionLog to record key random draws.
   - Playable acceptance: Starting a session with a seed and replaying the run reproduces identical outcomes for a seeded test scenario.
   - Notes: Make it trivial to replay a logged run by restoring seed + action sequence.

3) Implement append-only Actions Log endpoint and structured actions API
   - Goal: Provide an append-only actions log endpoint (GET /actions/log) and a structured action API to record player actions so runs can be reproduced from seed + action list.
   - Description: Implement an append-only, chronologically ordered ActionLog; expose GET /actions/log and an internal append API for the action handler to write atomic entries. Only player actions are logged (not internal token operations); combined with the initial seed, the action log is sufficient to reconstruct game state. ActionEntry contains only `seq` (index) and `payload` (the player action). No timestamp, actor, or other metadata is stored.
   - Playable acceptance: API returns chronologically ordered action entries and a replay test reconstructs state from seed + action log.
   - Notes: Make the ActionLog the canonical audit trail for player actions. Internal token operations (grant, consume, expire) are deterministic consequences of player actions and the seed, so they do not need explicit logging.

4) Implement canonical token list and lifecycle enforcement
   - Goal: Add the canonical token definitions (Insight, Renown, Refinement, Stability, Foresight, Momentum, Corruption, etc.) and lifecycle classes from vision.md.
   - Description: Implement token types in the `TokenType` enum with lifecycle on the `Token` struct. Tokens are created directly from TokenType (e.g. Token::persistent(token_type)); there is no separate token registry data structure. GameState.token_balances is the source of truth for token state.
   - Playable acceptance: Tests assert lifecycle transitions (grant, consume, expire) for at least three token types.
   - Notes: Keep the canonical token list authoritative and extensible via the TokenType enum.
    - Current token types (scope of Step 4): Health, Dodge, Stamina (basic survival tokens used in current combat).
    - Future token types (Step 4 onwards): Insight, Renown, Refinement, Stability, Foresight, Momentum, Corruption, Purity, and discipline-specific tokens.
    - Each token type declares its lifecycle on the Token struct (Permanent, PersistentCounter, FixedDuration, etc.).

5) Encounter replacement and scouting hooks (formerly "Add Area Decks")
   - Note: The structural AreaDeck work has been superseded by Library card location tracking (CardLocation enum with Deck/Hand/Discard). Encounter cards now use the same CardCounts as all other card types. This step focuses on the replacement-generation and scouting mechanics.
   - Goal: Implement the vision's replace-on-resolve behavior: resolved encounter cards are removed and replaced by freshly generated encounters with CardEffects; scouting biases replacement generation.
   - Description: Implement encounter consumption, replacement-generation (base type + CardEffects), and a simple CardEffect selection pipeline. Implement binding of encounter decks to the encounter instance (encounter deck, reward deck, CardEffect draws) and ensure any entry_cost for attempting an encounter is consumed/locked at start. All deck-bound draws, entry_cost consumes, and replacement-generation steps are recorded in the ActionLog.
   - Playable acceptance: Drawing and resolving an area encounter removes it from the Library hand and immediately creates a replacement entry; scouting-related parameters can bias replacement generation in deterministic tests.
   - Notes: Start with small CardEffect sets and deterministic replacement rules. ScoutingParams was deleted during cleanup and will need to be re-implemented here as part of the Library/GameState system rather than as a separate module.

6) Refactor combat into the library core (deterministic, logged) — COMPLETE

   - Note: CombatAction was a simple card-play struct { is_player, card_index }. CombatState is the pure-data combat representation. The old library::combat module and /combat/simulate endpoint have been fully removed; all combat resolution now uses GameState methods (start_combat, resolve_player_card, resolve_enemy_play, advance_combat_phase). CardDef struct has also been deleted.

   - Goal: Move combat resolution, deterministic start-of-turn draws, turn order, actions, and enemy scripts into the shared library, using the seeded RNG and writing a deterministic actions log.
   - Description: Define CombatState, enemy scripts, and resolution methods that produce an explicit, replayable combat log. Ensure start-of-turn mechanics (draws, tempo, and turn order) are deterministic and driven by the session RNG. Combat events are recorded via the ActionLog so every state change is auditable.
   - Playable acceptance: A single combat API produces deterministic combat results, reconciles card definitions and locations with the Library.
   - Notes: Combat is pure-data with minimal side-effecting entry points that only write to the action log.

7) Add the simple encounter play loop (pick -> fight -> replace -> scouting)
   - Goal: Support a single-playable encounter loop as described in the vision: pick an encounter, resolve it, perform the post-encounter scouting step, and repeat. 
   - Description: Implement /encounter/start, /encounter/step, /encounter/finish flows that use in-memory session state for now and write all events (including replacement and scouting decisions) to the ActionLog.
        - Remember that the action endpoint is the only endpoint allowed to change state. When the player plays an action (examples: pick an encounter, play a card, etc.), the game evaluates whether that changes any state (for example: move the combat one phase forward, conclude the combat, and go to the post-encounter scouting step, etc.). 
   - Playable acceptance: API user can draw an encounter, resolve combat to conclusion, perform a scouting post-resolution step that biases replacement, and the encounter Library cards update accordingly.
   - Notes: Ensure session can be replayed from seed + action log.
    - - Scouting parameters (preview count, CardEffect bias, pool modifier) are internal mechanics that influence encounter-generation deterministically during the scouting post-encounter step. They are not user-facing API endpoints but are controlled by the player's scouting action choices and token expenditures (Foresight, etc.).

7.5) Unify combat systems and remove old deck types — COMPLETE
   - Goal: Unify the two combat implementations so a single authoritative combat system resolves card effects and token lifecycles.
   - Description: Migrated resolve_card_effects to read from the Library and player state consistently. Removed legacy Deck, DeckCard, CardState from src/deck/ and player_data.cards, and test endpoints that relied on legacy deck CRUD. The library::combat module has been deleted; all combat logic lives in GameState methods.
   - Playable acceptance: A single combat API backed by GameState produces deterministic CombatStates and reconciles card definitions with the Library.
   - Minimal playable loop: After this step a very simple game loop exists: pick an encounter, play cards until one side has lost all HP, run a quick scouting phase (add the finished encounter card back into the encounter deck), then pick another encounter.

7.6) Flesh out combat and draw mechanics
   - Goal: Implement basic resource-card draw mechanics and encounter handsize rules to make pacing simple and deterministic.
   - Description: Resource cards are the only way to draw additional cards into hands: playing a resource card triggers draws onto one or more hands and is the primary way players gain cards to their hand. Enemies follow the same principle: certain enemy cards act as resource/draw cards that cause draws for their hands.
   - Encounter handsize & Foresight: The encounter handsize is controlled by the Foresight token (default starting value: 3). When an encounter is chosen it is moved to the discard pile and when the encounter is over cards are drawn until the encounter hand reaches the Foresight number of cards (this behavior applies to encounter hand management via Library CardCounts).
   - Enemy play behavior: On each enemy turn the enemy plays one random card matching the current CombatPhase (attack card during Attacking, defence during Defending, resource during Resourcing). After the player plays a card, the system automatically resolves the enemy's play and advances the combat phase.
   - Deck composition: Ensure starting decks for both players and enemies contain approximately 50% draw/resource cards so games have steady card-flow and pacing.
   - Current balance parameters: DrawCards per resource play is 1 attack, 1 defence, 2 resource (4 total) via CardEffectKind::DrawCards { attack: 1, defence: 1, resource: 2 }. These values significantly affect deck pacing and should be revisited as encounter complexity grows.
   - Playable acceptance: A minimal loop exists (pick -> fight -> scouting -> pick) with resource-card driven draws, Foresight-controlled encounter hands, enemy random play, and starting decks containing ~half draw cards.

7.7) Prepare CardEffects decks 
    - There is a player CardEffect "deck" and a EnemyCardEffect deck. 
    - By deck we mean a library representation of a deck. 
    - The enemy card effect deck is also in the library: even though all other enemy decks are on the encounter. 
    - No card in the player deck (besides the encounter) can have a CardEffect that is not present in the players CardEffect deck.
        - Same goes for the enemy cards Card effects, they all need to be represented in the enemy CardEffect "deck". 
    - These two decks will be used in the future: The enemy deck will be used during the post-encounter scouting phase to help generate new encounters for the encounter deck. The player CardEffect deck will be used during research to help generate new cards for the library. Details will be fleshed out in later steps.
    - So data wise every CardEffect on a card is a refference back to its "CardEffect"-card in the card effect "deck". 
        - When exposing the data on the endpoint the CardEffect on cards will just show the value and not the refference.
    - Playable acceptance: Library contains both player and enemy CardEffect decks; all card effects on player/enemy cards reference valid CardEffect deck entries (validated at initialization); GET /library/card-effects returns both decks.

8) Expand encounter variety (non-combat and hybrid encounters) — gathering first
   - Goal: Add gathering (Mining, Woodcutting, Herbalism, Fishing) and other encounter types that reuse the cards-and-tokens model and add discipline decks, and produce raw materials required for crafting.
   - Description: Implement node-based gathering encounters where discipline decks resolve the node (e.g., Mining uses Mining deck vs IronOre card) and produce raw/refined material tokens. Each discipline has its own durability pool ({Discipline}Durability) initialized at game start. Gathering encounters use EncounterPhase::InEncounter (same as combat) and the EncounterState enum dispatches to discipline-specific logic. Record material token grants in the ActionLog so crafting has a provable input history.
   - Player actions: Five canonical player actions: NewGame, EncounterPickEncounter, EncounterPlayCard, EncounterApplyScouting, EncounterAbort. EncounterAbort allows aborting non-combat encounters.
   - Replay: The replay system (replay_from_log) handles all 5 action types. Each new encounter type must extend the replay match arm. The EncounterAbort pattern (calling gs.abort_encounter() then transitioning phase) is a good template for future encounter-ending actions.
   - Playable acceptance: At least one gathering discipline is playable end-to-end, produces material tokens consumed by craft flows, and actions are routed via POST /action.
   - Notes: Ensure node encounter resolution follows the same remove-and-replace lifecycle: in this simple setup the just finished encounter is just added back to the deck again with no changes. The `/library/cards?card_kind=` endpoint must be extended for each new card kind. Consider auto-deriving from the CardKind enum in a future cleanup.
   - Scenario tests: update or add scenario tests in `tests/scenario_tests.rs` demonstrating the full gameplay loop with the new encounter type. Each new encounter type or gameplay feature from step 8 onwards should have at least one scenario test.
   - Current library card IDs (for reference in substeps):
     - 0-3: PlayerCardEffect entries (damage, shield, stamina, draw)
     - 4-7: EnemyCardEffect entries
     - 8: Attack card, 9: Defence card, 10: Resource card
     - 11: Combat encounter (Gnome)
     - 12: Mining card (Aggressive), 13: Mining card (Balanced), 14: Mining card (Protective)
     - 15: Mining encounter (Iron Ore)

8.1) Mining (gathering) — COMPLETE
   - Goal: First gathering discipline, establishing the EncounterState enum pattern for all future encounter types.
   - Description: Single-deck resolution. Player Mining deck with cards trading off ore_damage vs durability_prevent. Ore node has OreDeck with cards dealing 0-3 durability damage (skewed low). OreHealth tracked as encounter-scoped token in ore_tokens HashMap. MiningDurability initialized to 100 at game start, persists across encounters (NOT re-initialized per encounter).
   - Win: OreHealth ≤ 0 → grant rewards (Ore: 10, Token-keyed HashMap<Token, i64>). Loss: MiningDurability ≤ 0 → PlayerLost, no failure penalties (durability loss IS the penalty).
   - EncounterAbort: available for mining (marks as PlayerLost, no rewards/penalties).
   - BREAKING changes: /combat → /encounter, CombatState → EncounterState, CombatOutcome → EncounterOutcome, EnemyCardCounts → DeckCounts, EncounterPhase::Combat+Gathering → InEncounter.
   - Cleanup (docs/issues.md): is_finished removed, encounter_card_id mandatory, ore_tokens replaces ore_hp, Durability → MiningDurability, game-start durability init.
   - Playable acceptance: ✅ Mining end-to-end with 3 card types (Aggressive 5/0, Balanced 3/2, Protective 1/3), scenario tests, replay support.

8.2) Herbalism (gathering) — COMPLETE
   - Goal: Third gathering discipline with a UNIQUE mechanic (card-characteristic matching) that differentiates it from Mining/Woodcutting's damage-vs-durability template.
   - Description: The plant (enemy) starts with X cards on hand and does NOT draw more cards. Each enemy card has 1-3 characteristics from a small enum (e.g., Fragile, Thorny, Aromatic, Bitter, Luminous). Player plays Herbalism cards that target characteristics; playing a card removes all enemy cards that share at least one characteristic with the player's card. Player draws 1 Herbalism card per play.
   - Win condition: exactly 1 enemy card remains → that card is the reward, plus Plant tokens are granted.
   - Loss conditions: 0 enemy cards remain (over-harvested — player was too aggressive with broad-matching cards), OR HerbalismDurability ≤ 0 (durability depleted — durability cost applied via costs vec on play).
   - Tokens: HerbalismDurability (persistent, init 100 at game start), Plant (reward material token).
   - Key design notes:
     - The strategic tension is between playing narrow cards (remove fewer enemy cards, safer) vs broad cards (remove more, risk over-harvesting).
     - Enemy card characteristics create a puzzle: the player must read the remaining cards and choose which characteristics to target.
     - This is fundamentally different from Mining/Woodcutting (no HP-vs-durability loop) and creates the "knowledge and precision" feel from the vision.
   - Implementation checklist:
     1. Add PlantCharacteristic enum (Fragile, Thorny, Aromatic, Bitter, Luminous)
     2. Add CardKind::Herbalism { herbalism_effect: HerbalismCardEffect } with match_mode: HerbalismMatchMode, costs: Vec<GatheringCost>, gains: Vec<GatheringCost>
     3. Add HerbalismEncounterState with plant_hand (cards with characteristics), plant_deck (DeckCounts or direct Vec)
     4. Add EncounterState::Herbalism(HerbalismEncounterState)
     5. Add TokenType::HerbalismDurability, Plant
     6. Add Herbalism cards and encounter to initialize_library()
     7. Init HerbalismDurability to 100 in GameState::new()
     8. Implement resolve_player_herbalism_card (match characteristics, remove matching enemy cards, check win/loss)
     9. Dispatch in action handler and game_state resolution
     10. Update /library/cards?card_kind= filter for Herbalism
     11. Update replay_from_log
     12. Add scenario test
   - BREAKING changes: none (additive only).
   - New card IDs: 16 (Narrow Herbalism), 17 (Medium Herbalism), 18 (Broad Herbalism), 19 (Meadow Herb encounter).
   - Playable acceptance: ✅ Herbalism end-to-end with 3 card types (narrow/medium/broad characteristic targeting), 2 scenario tests (full loop + abort), replay support. All herbalism cards cost 1 durability. Plant hand randomized at encounter start using seeded RNG.

8.3) Woodcutting (gathering) — COMPLETE
   - Goal: Second gathering discipline with a UNIQUE mechanic (rhythm-based pattern matching) that differentiates it from Mining's damage-vs-durability template.
   - Description: Woodcutting is about hitting a rhythm for greater yields. There is NO enemy deck in this discipline.
     - Player Woodcutting cards have:
       - A `chop_type`: one of 5 types (LightChop, HeavyChop, MediumChop, PrecisionChop, SplitChop).
       - A `chop_value`: a number between 1-10.
       - Cards can have multiple types and multiple values, but initial cards have 1 of each.
       - Costs (durability, stamina, etc.) and gains expressed via `costs`/`gains` vecs. Durability depletion is a loss condition.
     - Turn flow: Player starts with hand size 5 and plays up to 8 cards total. Each time a card is played, 1 new Woodcutting card is drawn. All played cards are tracked.
     - After all cards are played (or the player chooses to stop early): evaluate the played cards for the best matching pattern and reward Lumber tokens accordingly.
     - Early stop: The player can choose to stop the woodcutting encounter after every card play. This is NOT an abort — the pattern of all cards played so far is still evaluated and rewards granted accordingly. Durability cost is only paid for the cards actually played. Pattern multipliers for better (rarer) patterns need to be significantly larger to justify playing more cards and risking more durability.
     - Patterns (poker-inspired but for up to 8 cards):
       - Implement many patterns at various reward tiers. Get inspired by poker hands but adapted for up to 8 cards.
       - Examples: all same type (flush), sequential values (straight), pairs/triples/quads of values, full house combinations, etc.
       - Only the single best pattern is used for reward calculation.
       - There should always be some reward — even the worst hand matches a simple pattern (e.g., "high card" equivalent).
       - Pattern rarity should have a significant impact on the multiplier to reward playing more cards.
     - Win: Always wins after all cards are played or the player stops early (the pattern determines reward amount).
     - Loss: WoodcuttingDurability ≤ 0 during play → PlayerLost, no rewards granted.
   - Tokens: WoodcuttingDurability (persistent, init 100 at game start), Lumber (reward material token).
   - EncounterAbort: available.
   - Key design notes:
     - The strategic tension is between playing cards that build toward better patterns vs. conserving durability.
     - No enemy deck means the player is solely focused on pattern construction from their hand.
     - The 8-card format (vs poker's 5) allows for richer pattern combinations.
   - Implementation checklist:
     1. Add ChopType enum (LightChop, HeavyChop, MediumChop, PrecisionChop, SplitChop)
     2. Add CardKind::Woodcutting { woodcutting_effect: WoodcuttingCardEffect } with chop_types: Vec<ChopType>, chop_values: Vec<u32>, costs: Vec<GatheringCost>, gains: Vec<GatheringCost>
     3. Add WoodcuttingEncounterState with played_cards: Vec tracking the 8 played cards, max_plays: 8
     4. Add EncounterState::Woodcutting(WoodcuttingEncounterState)
     5. Add TokenType::WoodcuttingDurability, Lumber
     6. Implement pattern evaluation engine (many patterns, poker-inspired, best-pattern-wins)
     7. Add Woodcutting cards and encounter to initialize_library()
     8. Init WoodcuttingDurability to 100 in GameState::new()
     9. Dispatch in action handler and game_state resolution
     10. Update /library/cards?card_kind= filter for Woodcutting
     11. Update replay_from_log
     12. Add scenario test
   - Playable acceptance: Woodcutting encounter playable end-to-end with pattern evaluation, produces Lumber tokens, scenario test passes.
   - BREAKING changes: none (additive only).
   - New card IDs: 20 (LightChop), 21 (HeavyChop), 22 (MediumChop), 23 (PrecisionChop), 24 (Oak Tree encounter).
   - New types: ChopType enum (5 variants), WoodcuttingCardEffect (chop_types, chop_values, costs, gains), PlayedWoodcuttingCard, WoodcuttingDef, WoodcuttingEncounterState.
   - New tokens: WoodcuttingDurability (persistent, init 100), Lumber (reward material).
   - NO enemy/tree deck — pattern matching mechanic is unique from Mining.
   - Poker-inspired pattern evaluation (13+ patterns, multipliers 1.0x–5.0x).
   - 2 scenario tests added (full loop + abort).
   - Playable acceptance: ✅ Woodcutting end-to-end with pattern evaluation, produces Lumber tokens, EncounterAbort supported.

8.4) Fishing (gathering) — COMPLETE
   - Goal: Fourth gathering discipline with a numeric card-subtraction mechanic that differentiates it from other gathering types.
   - Description: Each fishing encounter defines a `valid_range` (min, max), `max_turns`, and `win_turns_needed`. Each round the player plays a numeric card first, then the enemy (fish) plays a numeric card. The two values are subtracted: `result = (player_value - fish_value).max(0)`. If the result falls within the `valid_range` (inclusive), the turn counts as "won". Player draws 1 Fishing card per play. Costs (durability, stamina, etc.) are in the `costs: Vec<GatheringCost>` vector.
   - Win: player wins `win_turns_needed` rounds before `max_turns` are exhausted → grant Fish tokens.
   - Loss: `max_turns` exhausted without enough wins → PlayerLost. OR FishingDurability ≤ 0 → PlayerLost.
   - Tokens: FishingDurability (persistent, init 100 at game start), Fish (reward material token).
   - EncounterAbort: available.
   - Key design notes:
     - The tension is between playing high-value cards (better chance of exceeding the enemy but risk overshooting the valid range) vs low-value cards (may undershoot). The valid range creates a sweet spot the player must hit.
     - The max_turns limit creates urgency — the player has limited rounds to accumulate enough wins.
     - The fish deck (enemy) has cards with varying numeric values, creating uncertainty about what value the player needs to beat.
   - Implementation checklist:
     1. Add CardKind::Fishing { fishing_effect: FishingCardEffect } with values, costs: Vec<GatheringCost>, gains: Vec<GatheringCost>
     2. Add FishCard { value, counts: DeckCounts } for enemy fish cards
     3. Add FishingDef { valid_range, max_turns, win_turns_needed, fish_deck, rewards }
     4. Add FishingEncounterState with turns_won, max_turns, win_turns_needed, valid_range, fish_deck, rewards
     5. Add EncounterState::Fishing(FishingEncounterState)
     6. Add TokenType::FishingDurability, Fish
     7. Add Fishing cards and encounter to initialize_library()
     8. Init FishingDurability to 100 in GameState::new()
     9. Implement resolve_player_fishing_card (subtract values, range check, track wins, apply durability cost)
     10. Dispatch in action handler and game_state resolution
     11. Update /library/cards?card_kind= filter for Fishing
     12. Update replay_from_log
     13. Add scenario test
   - BREAKING changes: none (additive only).
   - New card IDs: 25 (Low Fishing, value 2), 26 (Medium Fishing, value 4), 27 (High Fishing, value 7), 28 (River Spot encounter).
   - New types: FishingCardEffect (values, costs, gains), FishCard (value, counts), FishingDef (valid_range_min/max, max_turns, win_turns_needed, fish_deck, rewards), FishingEncounterState.
   - New tokens: FishingDurability (persistent, init 100), Fish (reward material).
   - Enemy fish deck with 4 card variants (values 1, 3, 5, 7) — shuffled at encounter start.
   - Two loss conditions: max_turns exhausted without enough wins, OR FishingDurability ≤ 0.
   - 2 scenario tests added (full loop + abort).
   - Playable acceptance: ✅ Fishing end-to-end with card-subtraction, produces Fish tokens, EncounterAbort supported.

9.1) Major refactor: CardEffects range system ✅ Completed
   - Goal: Replace fixed numeric values on CardEffects with a min-max range system that allows card variation and makes future research/crafting systems meaningful.
   - Description: CardEffects define two values for each numeric parameter:
     - min: The minimum possible value for this effect.
     - max: The maximum possible value for this effect.
     When a concrete card is created (at library initialization or via crafting/research), a single fixed value is rolled between the CardEffect's min and max. This rolled value is stored on the concrete card and is always displayed to the player as a fixed number.
     When a card is played, its concrete fixed value is used directly — there is no per-play rolling.
   - Key constraints:
     - All rolls use the game seed for full reproducibility.
     - Concrete cards keep their reference to the CardEffect but also store their own fixed rolled value.
     - Overlapping CardEffects can be consolidated since the range system naturally covers variations.
     - Bump most numbers on cards, tokens, and encounters by a factor of ~100 (e.g., 1 → 100) to create interesting ranges. A value of 1 doesn't allow meaningful ranges (1-10 is too volatile), but 80-120 is fine.
   - This step is placed before Research and Crafting because those systems will add more CardEffects and concrete cards that benefit from the range system.
   - Playable acceptance: All existing cards use the range system. CardEffects define min/max ranges, concrete cards have a single rolled fixed value. All rolls are deterministic via the game seed. Scenario tests verify reproducibility.
   - Implementation results:
     - All cards use ConcreteEffect with rolled values from min-max ranges
     - All numeric values scaled ~100x (Health 2000, Durabilities 10000, etc.)
     - Deterministic rolling via game seed RNG
     - 35 cards total in library at time of implementation (was 29; count has since grown — see later steps)
     - All scenario tests pass with scaled values

> **Note on card counts:** Card counts in implementation results (e.g., "35 cards", "54 cards") are point-in-time records reflecting the count when that step was completed. They are not kept up to date as later steps add more cards.

9.2) CardEffects cost system ✅ Completed
   - Goal: Add an optional cost to CardEffects, creating a strategic cost-benefit dimension. The system must support multiple cost types and multiple costs per card from the start.
   - Description: Some CardEffects have one or more costs, each defined as a percentage range of the effect value:
     - Each cost entry specifies: a cost_type (e.g., Stamina) and a min-max percentage multiplier.
     - When a concrete card is created, each cost percentage is rolled from its range.
     - When played, each cost is calculated: rolled_effect_value × cost_percentage = cost_amount for that cost_type.
     - Example: Attack card with health effect range 100-500 and stamina cost 20-40%. Concrete card rolls value 200 with 25% cost. When played → 25% of 200 = 50 stamina cost.
   - Design rules:
     - Cards with a cost must be significantly better than non-cost equivalents.
     - In combat: at least one cost variation for each non-cost variation of attack and defence cards.
     - Stamina is the initial cost type and is shared across all disciplines.
     - The cost system is designed to be extensible: future cost types (e.g., Mana, Rations, discipline-specific currencies) can be added without structural changes. A single card may have multiple costs of different types.
     - Woodcutting and Mining cards can also have stamina costs (with a non-cost alternative for each).
     - Starting decks should mainly contain non-cost cards for ease of initial play.
   - Playable acceptance: Cost cards exist for combat, mining, and woodcutting. Cost calculations are deterministic. Starting decks are weighted toward non-cost cards. Scenario tests verify cost mechanics.
   - Implementation results:
     - Cost cards for Attack (id 31), Defence (id 32), Mining (id 33), Woodcutting (id 34)
     - Cost PlayerCardEffects (ids 29-30) with 30-50% Stamina cost ranges
     - Pre-validation prevents card consumption on insufficient resources
     - 4 new scenario tests verify cost mechanics
     - Starting decks: cost cards at deck:5 vs non-cost at deck:15
     - 1 cost Attack variant (vs 1 non-cost), 1 cost Defence variant (vs 1 non-cost), 1 cost Mining variant (vs 3 non-cost), 1 cost Woodcutting variant (vs 4 non-cost)
     - No cost Herbalism or Fishing variants (as per spec)
   - Known edge case — stuck encounter: When all remaining hand cards are cost cards and the player has no stamina (or other required resource), the encounter gets stuck (can't play any cards but encounter is still Undecided). Current workaround: player uses EncounterAbort. Future improvement: auto-detect when no playable cards remain and offer a forced pass or auto-loss. Affects both combat and gathering encounters.

9.3) MORE TOKENS and card variations ✅ Completed
   - Goal: Expand the range of good and bad cards by adding tokens, CardEffects, caps, and handsize management across all disciplines. This is the beginning of a greater work with adjustments expected in future steps.
   - Description:
     **Max handsize tokens:**
     - Every player deck that uses a hand (Attack, Defence, Resource, Mining, Herbalism, Woodcutting, Fishing) has a deck-specific token controlling max handsize (e.g., AttackMaxHand, DefenceMaxHand, MiningMaxHand, etc.). These tokens are respected during draws — no draw exceeds the handsize limit.
     - Every encounter that has an enemy hand size also has a token controlling that max handsize. In the future the player could have effects that impact enemy max handsize.
     - Initial handsize tokens should be set to reasonable defaults at game start.

     **Token caps (max thresholds):**
     - All CardEffects that grant a resource back (e.g., Stamina, Shield, Dodge, etc.) have a `cap` field (min/max range on the template, rolled to a concrete value). If adding the granted amount would exceed the cap, the value is clamped.
     - At least two types of resourcing cards in the player deck: one with a high cap (grants less resource) and one with a low cap (grants more resource). They can both reference the same CardEffect template with a range. This creates a strategic choice.

     **Multi-effect evaluation:**
     - If a card has multiple CardEffects, they are evaluated in isolation, first to last. This is relevant across many disciplines.
     - If a later CardEffect cannot pay its cost, the previous CardEffects still applied and the card play did not fail. A previous effect could grant the resource a later effect needs for its cost.

     **Generalized cost structure:**
     - All cards (except CardEffect templates) that have a cost store costs as `Vec<(TokenType, amount)>` so cost logic can be generalized.
     - Any card that cannot pay its cost cannot be played.
     - If the enemy picks a random card from hand, it should only try from cards where it can pay the cost. If it cannot pay the cost on any hand card, pick at random and pay as much as possible (even zero).
     - A future roadmap step will expand enemy AI and fix cost handling more thoroughly.

     **Fishing discipline expansions:**
     - Fishing player cards can have multiple CardEffects with multiple values (Vec<i64>); whichever value would "win the round" is chosen when the card is played. Initial deck cards have one CardEffect with one value each.
     - The fishing encounter state struct moves more fields to token-based setup.
     - New player fishing CardEffect: remove a "min value" token from the fish encounter (not below zero), affecting current and future turn win evaluation. Similar effect for increasing "max value" tokens. Both affect between 50-250 tokens (min/max range). Add a couple of cards with these effects to the player deck.
     - New player fishing CardEffect with a "cost" that does the reverse (narrows the valid range by increasing min or decreasing max). These cards have 3-5 values on a single CardEffect. Cost is determined by: (a) sum distance between the values (wider range = more options = higher cost), and (b) number of values (less impact than distance). Cost is defined as a range percentage — a very good card costs 200-350 stamina and a bad card 50-150. Calculate min-max percentage on the CardEffect to achieve this. Add one card to the player deck.
     - Add a "fish amount" token to the fish encounter.
     - Add a player CardEffect that increases the fish amount.
     - Add a player CardEffect with multiple values but decreases the fish amount (similar cost calculation as above).
     - Add a player CardEffect that gives significant stamina but has no values (a rest action while fishing).
     - If not already present, add a fishing action that costs stamina and has multiple values (similar cost calculation).

     **Herbalism discipline expansions:**
     - New player CardEffects:
       - A CardEffect that costs either Stamina or "reward amount" can have higher amounts on the effects below. The cost is a percentage range based on the benefit of the card.
       - A CardEffect that removes the plant type present on the most cards (limited to X plant types). Ties broken at random. More types removed is better.
       - A CardEffect that removes the plant type present on the least cards (limited to X plant types). Ties broken at random. More types removed is better.
       - A CardEffect with multiple types where only cards matching ALL types are removed ("and" based). At least 2 and at most all-minus-one plant types. More types is worse (2 types is best).
       - A similar CardEffect but "or" based (same costs).
     - Simple single-type CardEffect cards remain dominant in the initial deck. One of each special CardEffect on different cards in moderate versions.

     **Woodcutting expansions:**
     - Good/bad is straightforward: more numbers and patterns on a card = better. Sum of different numbers and patterns is the total "benefit."
     - CardEffect costs can be stamina and reward amount.
     - No-cost CardEffects have a card benefit between 1-4.
     - Cost CardEffects have a total card benefit of 5-15.
     - Initial deck is mainly no-cost cards with a couple of moderate cost/benefit cards.

     **Mining expansions:**
     - Good/bad is clear: high damage is always good, high defence is usually good.
     - Add cost-based CardEffects for stamina and rewards. Same mix as other disciplines.
     - Initial deck is mainly no-cost with some cost cards.
     - Note: A future roadmap step should improve mining gameplay since it currently is a simpler combat variant, which is a bit boring. But it is fine for now.

     **Combat expansions:**
     - Add a "milestone insight" reward token to all combat encounters. Add to the milestone roadmap step that starting a milestone encounter costs "milestone insight" tokens. Milestone insight (like all other rewards) is a token accumulated by the player.
     - Add CardEffects like all other disciplines: costs of stamina and rewards for greater effect. Deck is mainly non-cost cards.

   - Playable acceptance: All disciplines have expanded CardEffects with caps, costs, and handsize tokens. Multi-effect evaluation works correctly (first-to-last, partial success). Enemy AI respects cost affordability. Scenario tests cover new mechanics.
   - Implementation results:
     - Duration field on ChangeTokens (now split into GainTokens/LoseTokens) with TokenLifecycle, backward-compatible via serde defaults
     - Cap (cap_min/cap_max) and gain_percent (gain_min_percent/gain_max_percent) on GainTokens; rolled to concrete values, applied as clamp during token grants. LoseTokens uses positive min/max for amount to lose.
     - 10 max handsize tokens (AttackMaxHand, DefenceMaxHand, ResourceMaxHand, MiningMaxHand, HerbalismMaxHand, WoodcuttingMaxHand, FishingMaxHand, EnemyAttackMaxHand, EnemyDefenceMaxHand, EnemyResourceMaxHand) initialized to 10; enforced during draws without disrupting RNG sequence (later changed to 5 in post-9.3)
     - Multi-effect evaluation: effects evaluated sequentially per card, each pays its own cost, partial success allowed
     - Generalized cost structure: GatheringCost { cost_type, amount } vec on all gathering card types; merge_gathering_costs combines explicit costs with legacy inline fields (later simplified: dedicated fields removed, all costs in costs vec)
     - Autoloss: if all combat hand cards are unpayable (all effects have unaffordable costs), combat ends as PlayerLost (later extended to all disciplines in post-9.3)
     - MilestoneInsight token: 100 granted on combat PlayerWon
     - Fishing expansion: FishingCardEffect redesigned with values:Vec<i64>, gains/costs vecs; FishingRangeMin/FishingRangeMax/FishAmount tokens; 7 new cards (range widen x2, cost-narrow, fish amount+, multi-value fish decrease, rest, stamina cost)
     - Herbalism expansion: HerbalismMatchMode enum (Or{types}, And{types}, MostCommon{limit,types}, LeastCommon{limit,types}); gains/costs vecs; 4 new cards
     - Woodcutting expansion: gains/costs vecs; 5 new cards (SplitChop, dual-type, 3-type cost, 4-type cost, rest)
     - Mining expansion: gains/costs vecs; 3 new cards (high damage+protection cost, very high damage cost, rest)
     - 54 total library cards (was 35)
     - 7 new scenario tests covering MilestoneInsight, expansion card counts, max handsize initialization, fishing range tokens
     - Known: enemy cost handling not fully expanded (enemies don't check cost affordability for random picks); deferred to future enemy AI step

### Post-9.3 implementation (2026-03-02)
- BREAKING: `ChangeTokens` CardEffectKind split into `GainTokens` and `LoseTokens`. GainTokens has required cap_min/cap_max/gain_min_percent/gain_max_percent fields; LoseTokens has positive min/max (amount to lose). GainTokens cannot have a cost_type matching the gain token_type.
- BREAKING: `stamina_grant` field removed from all four discipline card effects (MiningCardEffect, HerbalismCardEffect, WoodcuttingCardEffect, FishingCardEffect). Replaced with `gains: Vec<GatheringCost>` for granting any token type on card play.
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
   - Implementation: Completed. Mining now uses a fully token-based system: `MiningCardEffect` has `costs`/`gains` (Vec<GatheringCost>) and `light_level_cap`; `OreCard` has `damages` (Vec<GatheringCost>). `MiningDef` has `initial_light_level` (300) and `ore_deck`. New token types: `MiningLightLevel`, `MiningYield`, `MiningPower` (all encounter-scoped, reset to 0 on encounter end). Yield formula: `mining_power × light_level / 100`. Conclude action: `EncounterConcludeEncounter` grants `min(stamina, yield)` Ore tokens. Loss conditions: `MiningDurability ≤ 0` or all hand cards unpayable. 8 player mining cards (power, light, rest varieties) + 1 encounter definition. All scenario tests pass.
   - **Post-cleanup summary (Step 9.5 post-cleanup pass):** This step included a significant cleanup pass affecting areas beyond mining: token restructuring (TokenType enum consolidation, encounter-scoped token migration from global token_balances to encounter_tokens), player death mechanic implementation (material reset, Health/Stamina restore, PlayerDeaths counter), EncounterConcludeEncounter standardized across all gathering disciplines (Mining, Herbalism, Woodcutting, Fishing), dynamic test ID migration (tests no longer rely on hardcoded card IDs), and documentation updates (vision.md/roadmap.md consolidation).

9.6) Crafting encounters and discipline
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
   - Stamina and Health tokens should be usable in CardEffects with costs within the crafting discipline, same deck mix as other discipline cards (mostly no-cost, some cost cards) in the initial deck.

10) Research encounters and card discovery
   - Goal: Implement Research as a first-class encounter type where players invest Insight tokens to discover and create new cards for their library.
   - **Implementation note**: Insight infrastructure is already partially in place — `MilestoneInsight` and `Insight` token types exist in the TokenType enum. The CardEffectKind::Insight variant and discipline tags remain to be implemented.
   - Description:
     - **CardEffect discipline tags**: Every CardEffect has a set of discipline tags (e.g., Combat, Mining, Herbalism, Woodcutting, Fishing) that determine which card types can use that effect. This enables effects to be shared across disciplines when appropriate.
       - Generalize the "Durability" card effects so they can be used across all gathering mechanics. When a durability card effect is played, the discipline context of the encounter determines which durability pool (MiningDurability, HerbalismDurability, WoodcuttingDurability, FishingDurability) is affected.
       - Review other CardEffects for similar generalization opportunities.
     - **Insight card effect**: Add a CardEffectKind::Insight variant that grants discipline-specific Insight tokens.
       - Can be added to every player card type (Attack, Defence, Resource, Mining, Herbalism, Woodcutting, Fishing, etc.).
       - Grants between 1-5 Insight tokens when the card is played.
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

11) Simple post-encounter scouting
   - Goal: Replace the current no-op scouting with a simple encounter-modification step that always happens after an encounter is concluded.
   - Description:
     - Always happens after an encounter is concluded. It always modifies the encounter card just concluded.
     - Any mention of tiers is postponed to the milestone step.
     - The player is presented with X options: generate X new encounters of the same type as the encounter just played.
     - Generate one encounter by:
       - For every encounter where one or more enemy decks are involved on the enemy side:
         - Keep the number of cards and card counts for each card.
         - Pick one card and reroll that card where "affix-pool size" is "CardEffect pool size" — each card has that amount of CardEffects.
         - Do not change the card type. Respect the CardEffect tags relative to the card type.
         - This is very similar to the player crafting step: just done repeatedly and with no player interaction.
         - If the deck had three cards and each card had 5 copies in total: only one of the three cards changes and it still has 5 copies.
         - It is okay to give the enemy CardEffects for a resource they cannot regain, because the enemy will always start with an initial amount of that resource.
       - If there are any numerical values (mainly initial tokens of the encounter): random change them in the range of -5% to +10%, so a good chance it will be tougher next time. Min should still be ≤ max if a min-max range exists.
     - Let the player choose which of the X encounters to replace the just-played encounter.
     - The player has to choose and cannot keep the just-played encounter.
     - When the player has chosen: replace the current player encounter card with the new encounter card and move to the next phase in the encounter.
   - Playable acceptance: After an encounter, X scouting options are generated and presented. Player must choose one. The selected encounter replaces the original in the Library. Scenario tests verify the scouting flow.
   - Notes: Keep initial choices small and data-driven. Up to this point all encounters just added the same encounter back into the encounter Library with no changes.

12) Finalize edge cases for a repeatable loop and concurrency
   - Goal: Make the loop robust: reshuffle/renew rules, player death and recovery, encounter exhaustion and replacement guarantees, and multi-session concurrency safety.
   - **Implementation note**: Player death and recovery is now implemented (Step 9.5 post-cleanup). When Health ≤ 0: gathering materials reset to 0, Health/Stamina restore to 1000, PlayerDeaths incremented, cards preserved. Remaining work: deck exhaustion/reshuffle edge cases, encounter replacement guarantees, concurrency controls, and save/load mechanics.
   - Description: Add tests for deck exhaustion/reshuffle, rules for encounter removal+replacement when decks empty, and concurrency controls for per-session Library encounter mutations.
   - **No file persistence on the server**: The server must not persist any game state to disk. All state lives in memory for the duration of a session.
   - **Save games via action log**: Players can query the full action log (`/actions/log`) and store it locally together with the game version code and the initial seed. This is the canonical "save game" format.
     - Expose a unique version code for each compiled game (e.g., a build hash or semantic version) via a dedicated endpoint (e.g., `/version`). This ensures saved action logs are replayed against the correct game binary.
     - Players can "load" any saved game by providing the action log, seed, and version to the server, which replays all actions to reconstruct the full game state.
     - Ensure this load/replay flow is achievable through existing or new endpoints.
     - **Action log size estimation**: Estimate how large an action log could grow in a typical long session (e.g., 500+ encounters). Evaluate whether loading via a single "load game" player action is viable or whether a dedicated POST endpoint that accepts the action log as a file payload is needed. Document the size thresholds and recommendation.
   - Playable acceptance: A session can play multiple encounters in sequence without violating invariants; action logs provide a full replay and tests pass. Save/load round-trips work correctly.
   - Notes: Add minimal instrumentation to spot-check correct replacement and token lifecycles.

13) Milestone encounters
   - Goal: Implement milestone encounters as the primary progression system tied to CardEffects.
   - **Implementation note**: The `PlayerDeaths` token (incremented on each death) could factor into milestone difficulty scaling — e.g., milestones become harder after more deaths, or certain milestones unlock only after surviving a death threshold.
   - Description:
     - Each interesting CardEffect has a corresponding milestone encounter.
     - When a milestone is beaten, a more powerful version of the milestone is created that rewards the next tier of that CardEffect.
     - Some milestones reward tokens like "max hand size" increases.
     - Some milestones require beating other milestones first to obtain a "token key" prerequisite.
     - A full list of CardEffects eligible for milestones will be compiled when this step begins.
   - Playable acceptance: Players can play milestone encounters, earn rewards, unlock higher tiers, and prerequisite chains work correctly.
   - Notes: Start with a small set of milestones to prove the system before expanding to all CardEffects.

14) Configuration externalization
   - Goal: Move all game configuration into JSON files loaded at compile time.
   - **Implementation note**: The current card registration pattern is hardcoded in Rust discipline modules (`src/library/disciplines/combat.rs`, `mining.rs`, `herbalism.rs`, `woodcutting.rs`, `fishing.rs`). Each module calls `register_*` functions to add CardEffect templates and cards to the Library. This is the specific code being externalized into JSON configuration files.
   - Description:
     - All initial library cards, tokens, and other configuration are defined in JSON files.
     - A new `configurations/` folder at the repository root organizes config by discipline and a general section.
     - Structure: `configurations/general/`, `configurations/mining/`, `configurations/herbalism/`, `configurations/woodcutting/`, `configurations/fishing/`, `configurations/combat/`, `configurations/crafting/`, etc.
     - Configuration is baked into the compiled binary — a compiled game cannot change these values, but developers can adjust them before compiling.
   - Playable acceptance: All card definitions, initial token values, and encounter parameters come from JSON config files. Changing a config file and recompiling produces a game with the updated values.
   - Notes: This enables designers to tweak game balance without touching Rust code.

15) UX polish, documentation, tools for designers, and release
   - Goal: Finalize API docs (OpenAPI/Swagger), provide a sample client that drives the full loop, and ship a release with clear design docs for authors. Anyone should be able to play the game solely with the exposed documentation.
   - Description: Add designer tooling for encounter/CardEffect creation, telemetry for balancing, and example playthroughs demonstrating reproducibility from seed+action-log.
   - **New-player tutorial endpoint**: Expose a tutorial for new players at an endpoint (e.g., `/docs/tutorial`), linked from the README.md as a running-server URL. The tutorial should walk a new player through their first game session.
   - **Rich OpenAPI specification**: The OpenAPI/Swagger documentation should go beyond simple endpoint specifications — it should convey the general mentality and strategic purpose of each discipline and action. Descriptions should explain *why* a player would use an endpoint, not just *what* it does.
   - **Hints and strategies page**: Expose a "hints" endpoint (e.g., `/docs/hints`) with good strategies, common pitfalls, and tips for each discipline. This helps players discover interesting gameplay patterns.
   - **Self-sufficient documentation goal**: The combined documentation (tutorial + OpenAPI + hints) must be comprehensive enough that anyone can play the game solely with this documentation, without needing external guides or source code access.
   - Playable acceptance: A developer can run a reproducible session from seed and action-log and follow README to play a full campaign.
   - Notes: Tag a release and include release notes linking vision to implemented features.

16) Balancing setup
   - Goal: Establish automated tools and processes for balance testing and tuning.
   - Description:
     - Define balancing goals for each discipline.
     - Build a mutating runner that tries different strategies and documents whether they are all viable for reaching specific goals, ensuring multiple paths to victory.
     - Scope initially to balancing each discipline individually: verify multiple strategies per discipline are viable and interesting.
     - Define expected outcomes per encounter per tier and expected fail/success rates.
     - Run the balancing tools and collect data.
     - Analyze data and make adjustments.
   - Playable acceptance: Automated balance runners produce data showing strategy viability across disciplines. Results inform configuration adjustments.
   - Notes: Keep the runner deterministic (seeded) for reproducible balance analysis.

Ideas and future possibilities
------------------------------
Implement Trading and Merchants (MerchantOffers + Barter workflow)
   - Goal: Model merchants as decks (MerchantOffers, Barter) and deterministic merchant interactions that mirror vision.md's barter mechanics.
   - Description: Implement MerchantOffers deck and Barter deck support. Merchant interactions deterministically draw a MerchantOfferPool, present offers, then draw MerchantBargainDraw barter cards and allow choosing up to MerchantLeverage to modify the selected offer. Barter cards can change offered_token, requested_token, rate, fee, or attach conditions. All draws, choices, and resulting token transfers are recorded in the ActionLog so merchant interactions are reproducible.
   - Playable acceptance: A /merchant/{id}/visit endpoint returns deterministic offers derived from the session seed; applying barter choices updates player tokens and is recorded.
   - Notes: Start with a single merchant and simple barter cards; expand to dynamic merchant inventory later.
   - Note: We need to determine both whether trading is needed for the game loop and what the unique gameplay of the barter mini-game should be before implementing.

Implementation guidelines and priorities
--------------------------------------
- Validate alignment with docs/vision.md for every milestone; require one explicit mapping note in PRs describing which lines in vision.md the work satisfies.
- Keep core logic pure and testable; make side effects pluggable and thin wrappers to the ActionLog.
- Prioritize deterministic behavior and reproducibility from the start.
- Prefer data-driven content formats (deck files, CardEffect tables) so designers can author content without code changes.
- Try to migrate tests away from test endpoints and use only public endpoints. Only use test endpoints temporarily if it is not possible to run the test without them; the expectation is that a later point in the roadmap will make any test endpoint redundant. 
- Test migration status: `tests/scenario_tests.rs` uses only production endpoints and serves as the model for new tests. Track which test files still depend on /tests/* endpoints and target full migration as features make test endpoints redundant.
- Large steps should be split into numbered sub-steps (e.g., 9.3.1, 9.3.2) for better progress tracking. Step 9.3 had 13 sub-steps and 12 commits — future steps of similar magnitude benefit from finer granularity in the roadmap.
- Older step descriptions may reference field names that were later refactored (e.g., `durability_cost`, `stamina_cost`, `modify_range_min`). These are historical records describing the state at time of implementation and should not be updated retroactively.

How to use this roadmap
-----------------------
- Use steps as milestones for sprints or PRs. Each PR should implement a step's minimum playable acceptance criteria and include tests that reproduce behaviour from a seed.
- After completing each step, add at least one integration test that exercises the end-to-end playable loop for that milestone and verifies action-log replay.
- Iterate on balance and UX only after mechanics are stable and auditable.

Appendix: Minimum ticket examples for the first 8 steps
-----------------------------------------------------
- Refactor library: Extract Card/Hand/Token/Library into lib crate, add unit tests for move/draw/reshuffle, and implement TokenType enum for token definitions.
- RNG: Add seeded RNG, deterministic derive API, and replay helper for restoring runs.
- Token lifecycles: Implement Insight/Foresight/Renown/Refinement/Stability and action-log recording for lifecycle events.
- Encounter tracking: Track encounter cards via Library CardCounts (deck/hand/discard), implement draw/resolve/replace and CardEffect replacement pipeline.
- Combat refactor: Implement CombatState and resolution using seeded RNG and output deterministic logs recorded to the ActionLog.
- Encounter loop: Implement the encounter loop via POST /action (pick encounter, play cards, advance phases, scouting), include replacement and scouting as part of the lifecycle.
- Research: Implement CardEffect discipline tags, Insight card effect, research encounter with choose/progress actions, and deterministic card generation recorded to the ActionLog.
- Merchants: Implement MerchantOffers and Barter decks, deterministic visits, and barter flows recorded to the ActionLog.

CardEffect ideas (future)
-------------------------
A lot of these could be introduced with a Milestone boss encounter or as progression rewards.

- **Life steal:** All disciplines could have a "life steal" CardEffect that converts a portion of the effect's value into health or resource recovery.
- **Max handsize milestones:** Some milestones can increase the max handsize in a specific discipline, providing a permanent progression reward.
- **Merchant rare deals:** Some merchants can offer rare interesting deals where permanent tokens can be exchanged for other permanent tokens (e.g., trade max handsize in woodcutting for max handsize in crafting).
- **Herbalism guard token:** A "guard" token that protects plant cards in some way but only exists for a very short period. Not a guaranteed win condition — for example, the next card play leaves half of the cards that would have been removed, chosen at random.
- **Card forgetting:** A CardEffect that lets the player forget (permanently remove) a full card including all copies, as long as all copies are in the library and not in deck/hand/discard. More powerful effect the more cards are crafted; still a good effect even without crafted cards. This is a way to clean up the library of old unused cards. Requires implementing empty entries in the library vector with ID reuse — critical that existing cards keep their IDs.
- **Magic CardEffects:** Add magic-themed CardEffects for all disciplines, potentially gated behind research or milestones.
- **Cooking mechanic:** Expand the rest encounter with a cooking sub-system. With rest now using library cards and rest tokens, cooking could interact with the rest token system — e.g., cooking grants additional rest tokens, modifies rest card effects, or introduces new rest card types. Creates demand for Fish and Plant tokens.
- **Faction expansion of milestones:** Expand milestones into a faction mechanic with more sense of player choices and possibly a faction discipline deck.
- **Scouting expansion:** Expand the scouting step to give the user more choice rather than deterministic difficulty increases. It should be possible to shape the difficulty (and rewards) of an encounter and leave the nature of the enemy somewhat random. Maybe choosing 1 out of 3 options. Adding a Scouting discipline deck when a good mechanic is figured out.

Code architecture improvements (future)
----------------------------------------
- **Extend HasDeckCounts to player library cards:** `LibraryCard` uses `CardCounts` (with an extra `library` field) instead of `DeckCounts`. Consider a broader `HasCounts` trait hierarchy or unifying `CardCounts` and `DeckCounts` so player deck draw/shuffle operations can also use generic functions, further reducing duplication in `draw_player_cards_of_kind`.
- ~~**Generalize ore play-random in mining.rs:**~~ Resolved — mining redesign removed OreHealth and enemy damage, using token-based card effects and `deck_play_random` patterns throughout.
- ~~**Fix pre-existing test failures:**~~ Resolved — `test_play_attack_card_kills_enemy`, `test_play_defence_card_adds_tokens` (resolve_play_tests.rs), and `test_player_kills_enemy_and_combat_ends` (flow_tests.rs) now discover card IDs dynamically via API.
- **Statistical testing for woodcutting patterns:** The woodcutting multiplier rebalance was calibrated using an external Python Monte Carlo simulation. Consider adding a Rust-native test or benchmark that validates pattern probabilities are within expected ranges, ensuring future deck composition changes don't silently break the probability assumptions.

Known game design gaps (future)
--------------------------------
- ~~**Health initialization gap:**~~ Resolved — Health token is now set to 1000 at game start in `new_with_rng()`.
- **Rest token progression:** Currently 1–2 rest tokens per encounter are hardcoded. Future upgrades could grant more rest tokens per encounter, making rest more powerful as the game progresses. This parallels how other disciplines improve through card effects and MaxHand increases. Could be gated behind milestones or research.

Technical debt and cleanup tracking
------------------------------------
Items that accumulate during development and should be addressed periodically:

- **Hardcoded card IDs in tests:** Tests that use hardcoded card IDs (e.g., `card_id: 11`) break when card registration order changes. Prefer dynamic ID discovery via API queries. Scenario tests now use dynamic IDs; some older flow/resolve tests may still have fragile ID references.
- **Outdated type names in docs:** As types are renamed or restructured (e.g., `GatheringCost` → `TokenAmount`, `durability_cost` → `costs`), documentation and comments may lag behind. Periodic sweeps should update stale references.
- **Undocumented encounter patterns:** When new encounter types or patterns are added, they should be documented in vision.md (encounter template section) and tested with scenario tests. Gaps between implementation and documentation accumulate as tech debt.
- **unwrap() calls in production code:** Currently 5 `unwrap()` calls exist in `src/action/persistence.rs` and `src/library/game_state.rs`. These should be replaced with proper error handling over time per the no-unwrap policy.
- **Test endpoint dependency:** Some tests in `tests/flow_tests.rs` and `tests/resolve_play_tests.rs` still use `/tests/combat` instead of production endpoints. Track and migrate these as features make test endpoints redundant.

