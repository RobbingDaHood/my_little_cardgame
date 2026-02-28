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
   - Goal: Add a single-game RNG and a deterministic scheduler that all systems use (deck shuffles, encounter generation, affix rolls, combat decisions where non-deterministic choices exist).
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
   - Goal: Implement the vision's replace-on-resolve behavior: resolved encounter cards are removed and replaced by freshly generated encounters with affixes; scouting biases replacement generation.
   - Description: Implement encounter consumption, replacement-generation (base type + affixes), and a simple affix pipeline. Implement binding of encounter decks to the encounter instance (encounter deck, reward deck, modifier pulls) and ensure any entry_cost for attempting an encounter is consumed/locked at start. All deck-bound draws, entry_cost consumes, and replacement-generation steps are recorded in the ActionLog.
   - Playable acceptance: Drawing and resolving an area encounter removes it from the Library hand and immediately creates a replacement entry; scouting-related parameters can bias replacement generation in deterministic tests.
   - Notes: Start with small affix sets and deterministic replacement rules. ScoutingParams was deleted during cleanup and will need to be re-implemented here as part of the Library/GameState system rather than as a separate module.

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
    - - Scouting parameters (preview count, affix bias, pool modifier) are internal mechanics that influence encounter-generation deterministically during the scouting post-encounter step. They are not user-facing API endpoints but are controlled by the player's scouting action choices and token expenditures (Foresight, etc.).

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
   - Loss conditions: 0 enemy cards remain (over-harvested — player was too aggressive with broad-matching cards), OR HerbalismDurability ≤ 0 (durability depleted — each player card has a durability_cost applied immediately on play).
   - Tokens: HerbalismDurability (persistent, init 100 at game start), Plant (reward material token).
   - Key design notes:
     - The strategic tension is between playing narrow cards (remove fewer enemy cards, safer) vs broad cards (remove more, risk over-harvesting).
     - Enemy card characteristics create a puzzle: the player must read the remaining cards and choose which characteristics to target.
     - This is fundamentally different from Mining/Woodcutting (no HP-vs-durability loop) and creates the "knowledge and precision" feel from the vision.
   - Implementation checklist:
     1. Add PlantCharacteristic enum (Fragile, Thorny, Aromatic, Bitter, Luminous)
     2. Add CardKind::Herbalism { herbalism_effect: HerbalismCardEffect } with target_characteristics: Vec<PlantCharacteristic>, durability_cost: i64
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
       - A `durability_cost`: a fixed small cost (like herbalism, around 1). Durability depletion is a loss condition.
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
     2. Add CardKind::Woodcutting { woodcutting_effect: WoodcuttingCardEffect } with chop_types: Vec<ChopType>, chop_values: Vec<u32>, durability_cost: i64
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
   - New types: ChopType enum (5 variants), WoodcuttingCardEffect (chop_types, chop_values, durability_cost), PlayedWoodcuttingCard, WoodcuttingDef, WoodcuttingEncounterState.
   - New tokens: WoodcuttingDurability (persistent, init 100), Lumber (reward material).
   - NO enemy/tree deck — pattern matching mechanic is unique from Mining.
   - Poker-inspired pattern evaluation (13+ patterns, multipliers 1.0x–5.0x).
   - 2 scenario tests added (full loop + abort).
   - Playable acceptance: ✅ Woodcutting end-to-end with pattern evaluation, produces Lumber tokens, EncounterAbort supported.

8.4) Fishing (gathering) — COMPLETE
   - Goal: Fourth gathering discipline with a numeric card-subtraction mechanic that differentiates it from other gathering types.
   - Description: Each fishing encounter defines a `valid_range` (min, max), `max_turns`, and `win_turns_needed`. Each round the player plays a numeric card first, then the enemy (fish) plays a numeric card. The two values are subtracted: `result = (player_value - fish_value).max(0)`. If the result falls within the `valid_range` (inclusive), the turn counts as "won". Player draws 1 Fishing card per play. Every card played has a small `durability_cost` applied to FishingDurability.
   - Win: player wins `win_turns_needed` rounds before `max_turns` are exhausted → grant Fish tokens.
   - Loss: `max_turns` exhausted without enough wins → PlayerLost. OR FishingDurability ≤ 0 → PlayerLost.
   - Tokens: FishingDurability (persistent, init 100 at game start), Fish (reward material token).
   - EncounterAbort: available.
   - Key design notes:
     - The tension is between playing high-value cards (better chance of exceeding the enemy but risk overshooting the valid range) vs low-value cards (may undershoot). The valid range creates a sweet spot the player must hit.
     - The max_turns limit creates urgency — the player has limited rounds to accumulate enough wins.
     - The fish deck (enemy) has cards with varying numeric values, creating uncertainty about what value the player needs to beat.
   - Implementation checklist:
     1. Add CardKind::Fishing { fishing_effect: FishingCardEffect } with value, durability_cost
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
   - New types: FishingCardEffect (value, durability_cost), FishCard (value, counts), FishingDef (valid_range_min/max, max_turns, win_turns_needed, fish_deck, rewards), FishingEncounterState.
   - New tokens: FishingDurability (persistent, init 100), Fish (reward material).
   - Enemy fish deck with 4 card variants (values 1, 3, 5, 7) — shuffled at encounter start.
   - Two loss conditions: max_turns exhausted without enough wins, OR FishingDurability ≤ 0.
   - 2 scenario tests added (full loop + abort).
   - Playable acceptance: ✅ Fishing end-to-end with card-subtraction, produces Fish tokens, EncounterAbort supported.

8.5) Refined gathering encounters
   - Goal: Evolve all simplified gathering disciplines (8.1-8.4) toward richer gameplay with tiered rewards, difficulty scaling, and strategic card choices.
   - Description: Add the features that differentiate the vision's end-state from the current simple implementations:
     - Tiered reward tokens: Each gathering discipline produces tiered rewards (e.g., Lumber T1, Lumber T2, Lumber T3; Ore T1, Ore T2, Ore T3; etc.). Tier 1 is the default reward from the base encounter.
     - Tier-increasing player card effects: Add card effects that increase the encounter's reward tier when played. These cards make the encounter more difficult through gameplay-involved mechanics (NOT simply by removing durability). Tier 2 should be moderately hard to achieve; Tier 3 should be very hard. The difficulty increase should be organic to each discipline's mechanics.
     - Insight tokens with three tiers: Insight T1, T2, T3 — generated by discipline-specific insight card effects. Higher tiers require harder gameplay to earn.
     - Initial decks have very few cards with tier-increase effects. Players acquire more through crafting/research later.
     - Stamina replaces Rations as the cost currency for encounter boosts and card costs across all disciplines.
     - Tiers could be expanded beyond 3 later, but initially just three.
   - Deferred to later (milestone step or beyond):
     - Combos and Momentum mechanics.
     - Affix modifiers on encounter cards (deferred from scouting step as well).
   - Additional notes from implementation experience:
     - Woodcutting pattern engine could be expanded: add combo multipliers for chaining patterns across encounters, add SplitChop card to starting deck.
     - All 4 gathering disciplines (Mining, Herbalism, Woodcutting, Fishing) are now implemented with the same EncounterState enum pattern. The pattern is confirmed reusable: each discipline adds a new variant to EncounterState, EncounterKind, and CardKind, then plugs into the existing action dispatch and replay infrastructure.
     - Four distinct mechanical templates exist:
       1. Damage-vs-durability loop (Mining): player deals damage to node HP while node deals durability damage; mutual draw each turn.
       2. Card-characteristic matching (Herbalism): card matching to narrow the enemy hand; no enemy draws.
       3. Poker-like pattern building (Woodcutting): play cards to build patterns; no enemy deck; degree-of-success rather than binary win/lose.
       4. Card-subtraction with valid-range targeting (Fishing): player and enemy both play numeric cards; result must land within target range.
     - Consider whether any of these can share infrastructure or whether each discipline's resolution should remain fully independent.
   - This step bridges the gap between the simple playable versions and the richer encounter experience.
   - Playable acceptance: At least one gathering discipline demonstrates tiered rewards (T1/T2/T3), tier-increasing card effects with gameplay-involved difficulty increase, and insight token generation across tiers. Initial decks have limited tier-increase cards. Scenario tests verify the tiered mechanics.

8.6) Gathering balance pass
   - Goal: After all 4 gathering disciplines are implemented (8.1-8.4), perform a dedicated balance and tuning pass.
   - Description:
     - Normalize durability init values (all currently 100 — may need differentiation per discipline).
     - Balance reward token amounts across disciplines (Ore, Plant, Lumber, Fish should have comparable value-per-encounter).
     - Tune encounter card counts and difficulty distributions.
     - Add cross-discipline scenario tests (e.g., mine then herbalism then combat in sequence).
   - Playable acceptance: Cross-discipline scenario tests pass. Reward amounts feel balanced across disciplines. Durability values create meaningful multi-encounter arcs.

9.1) Major refactor: CardEffects range system
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

9.2) CardEffects cost system
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

9.3) Rest encounter
   - Goal: Add a rest encounter type that allows stamina recovery and creates a meaningful pacing mechanic.
   - Description: A new encounter type where the player picks a rest benefit card.
     - The starting encounter deck should have ~20% rest encounters.
     - A rest card effect is defined with a wide range (min-max) using the Step 9.1 range system.
     - 5 different rest cards are rolled from this effect at game initialization.
     - Each of those 5 cards has 5 copies in the rest deck (25 total cards).
     - When the encounter starts: draw 5 rest cards from the deck and present them as choices.
     - The player picks 1 card. That card's effect (e.g., stamina recovery) takes effect immediately and the encounter is won.
     - To start, rest cards only provide stamina recovery. Future iterations may add health recovery, durability repair, or other benefits.
   - Implementation checklist:
     1. Add EncounterKind::Rest { rest_def: RestDef } with rest deck and rewards.
     2. Add RestCardEffect with stamina_recovery (using range system from 9.1, or fixed values if 9.1 is not yet implemented).
     3. Add CardKind::Rest { rest_effect: RestCardEffect } for rest action cards.
     4. Add EncounterState::Rest(RestEncounterState) with drawn hand of 5 cards.
     5. Implement EncounterPlayCard for rest: apply chosen card's effect, mark encounter as won.
     6. Add rest encounters to initialize_library() (~20% of encounter deck).
     7. Add scenario test.
   - Playable acceptance: Rest encounters appear in the encounter deck, player draws 5 rest cards and picks 1, stamina is recovered, encounter completes as PlayerWon. Scenario test passes.

9) Crafting encounters and discipline
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

10) Add Research/Learning and Modifier-Deck pipeline
   - Goal: Implement Research/Learning as first-class encounters that produce Insight, Variant-Choice, and Affix-Picks and generate Library variants via a Modifier Deck workflow.
   - Description: Add a ResearchDeck and ModifierDeck types; support presenting X research candidates (driven by tokens like "max research choice"), selecting one to start, and resolving research by drawing modifier cards and attaching up to Y affixes (Y from Affix-Picks). Random draws and affix numeric rolls use the global seed and all steps are recorded in the ActionLog. Completed variants are added to the Library (never directly to player decks).
   - Playable acceptance: A Research flow endpoint presents candidates, accepts a selection, resolves modifier draws deterministically using the session seed, produces a Library variant, and logs every decision/draw.
   - Notes: Start with a small ResearchDeck and ModifierDeck; ensure variant generation and replay are deterministic and auditable. ModifierDeck entries come primarily from Milestones/Challenge rewards and unique modifier acquisition; implement rules to replenish ResearchDeck candidates after resolution so research remains a continuous pipeline.

11) Add post-encounter scouting choices (vision-driven)
   - Goal: Present scouting choices as a post-resolution step that influence replacement-generation parameters (preview counts, affix biases, candidate pools) and grant Foresight/related tokens.
   - Description: Implement ScoutChoice objects and a deterministic application that updates replacement parameters for encounter generation (using Library CardCounts). Record scouting decisions and effects in the ActionLog.
   - Playable acceptance: After an encounter, API returns scouting choices; making a choice updates the replacement-generation seed/parameters and is reflected in the next replacement card deterministically.
   - Notes: Keep initial choices small and data-driven (e.g., +1 Foresight, increase affix-pool size).
    - Up to this point then all encounters just added the same encounter back into the encounter Library: no changes. 

12) Implement Trading and Merchants (MerchantOffers + Barter workflow)
   - Goal: Model merchants as decks (MerchantOffers, Barter) and deterministic merchant interactions that mirror vision.md's barter mechanics.
   - Description: Implement MerchantOffers deck and Barter deck support. Merchant interactions deterministically draw a MerchantOfferPool, present offers, then draw MerchantBargainDraw barter cards and allow choosing up to MerchantLeverage to modify the selected offer. Barter cards can change offered_token, requested_token, rate, fee, or attach conditions. All draws, choices, and resulting token transfers are recorded in the ActionLog so merchant interactions are reproducible.
   - Playable acceptance: A /merchant/{id}/visit endpoint returns deterministic offers derived from the session seed; applying barter choices updates player tokens and is recorded.
   - Notes: Start with a single merchant and simple barter cards; expand to dynamic merchant inventory later.

13) Finalize edge cases for a repeatable loop and concurrency
   - Goal: Make the loop robust: reshuffle/renew rules, player death and recovery, encounter exhaustion and replacement guarantees, and multi-session concurrency safety.
   - Description: Add tests for deck exhaustion/reshuffle, rules for encounter removal+replacement when decks empty, and concurrency controls for per-session Library encounter mutations.
   - Playable acceptance: A session can play multiple encounters in sequence without violating invariants; action logs provide a full replay and tests pass.
   - Notes: Add minimal instrumentation to spot-check correct replacement and token lifecycles.

14) Add persistent player progression, library-driven deck-building, and upgrades
   - Goal: Add persistence for player state, deck composition, tokens, and a simple upgrade/shop flow; allow adding Library cards to player decks subject to constraints.
   - Description: Implement file-backed or DB-backed player-store, endpoints for deck-editing, and a small upgrade flow that consumes tokens to unlock Library cards or add copies to decks.
   - Playable acceptance: Player progress persists across runs; players can add Library items to decks and token spends are recorded in the ActionLog.
   - Notes: Keep persistence implementation pluggable and optional for tests.


15) Add resource management, camp mechanics, and short-term tokens
   - Goal: Add resources (Rations, Durability, Exhaustion) and camp actions (rest, craft, short-scout) that consume or restore resources and interact with scouting/replacement.
   - Description: Implement resource counters with lifecycle types and camp endpoints that modify player/discipline state and log actions.
   - Playable acceptance: Camp endpoint effects are visible in subsequent encounters and token lifecycles behave as specified.
   - Notes: Keep resource pools constrained to create meaningful trade-offs.

16) Implement varied enemy AI, conditional card effects, and targeting
   - Goal: Add enemy behaviors and richer card effect syntax (conditions, triggers, target selectors) while keeping deterministic resolution and action logging.
   - Description: Provide behavior profiles for enemies and expand the card schema; cover interactions with momentum, status tokens, and conditional triggers.
   - Playable acceptance: New cards and behaviors are playable and deterministic given the same seed; unit tests cover conditional resolution.
   - Notes: Keep scripting sandboxed and composable.

17) Introduce persistent world/meta-progression and milestone systems
   - Goal: Track area clears, milestones, and unlocks; implement milestone rewards that grant Variant-Choice/Affix-Picks and long-lived tokens.
   - Description: Persist campaign state, add milestone flows, and ensure progression tokens are granted and recorded in the ActionLog.
   - Playable acceptance: A simple campaign unlock path is playable and tokens/unlocks persist across sessions.
   - Notes: Provide tools for resetting campaigns for testing.

18) UX polish, documentation, tools for designers, and release
   - Goal: Finalize API docs (OpenAPI/Swagger), provide a sample client that drives the full loop, and ship a release with clear design docs for authors.
   - Description: Add designer tooling for encounter/affix creation, telemetry for balancing, and example playthroughs demonstrating reproducibility from seed+action-log.
   - Playable acceptance: A developer can run a reproducible session from seed and action-log and follow README to play a full campaign.
   - Notes: Tag a release and include release notes linking vision to implemented features.

Implementation guidelines and priorities
--------------------------------------
- Validate alignment with docs/vision.md for every milestone; require one explicit mapping note in PRs describing which lines in vision.md the work satisfies.
- Keep core logic pure and testable; make side effects pluggable and thin wrappers to the ActionLog.
- Prioritize deterministic behavior and reproducibility from the start.
- Prefer data-driven content formats (deck files, affix tables) so designers can author content without code changes.
- Try to migrate tests away from test endpoints and use only public endpoints. Only use test endpoints temporarily if it is not possible to run the test without them; the expectation is that a later point in the roadmap will make any test endpoint redundant. 
- Test migration status: `tests/scenario_tests.rs` uses only production endpoints and serves as the model for new tests. Track which test files still depend on /tests/* endpoints and target full migration as features make test endpoints redundant.

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
- Encounter tracking: Track encounter cards via Library CardCounts (deck/hand/discard), implement draw/resolve/replace and affix replacement pipeline.
- Combat refactor: Implement CombatState and resolution using seeded RNG and output deterministic logs recorded to the ActionLog.
- Encounter loop: Implement the encounter loop via POST /action (pick encounter, play cards, advance phases, scouting), include replacement and scouting as part of the lifecycle.
- Research: Implement ResearchDeck + ModifierDeck pipeline and deterministic variant generation recorded to the ActionLog.
- Merchants: Implement MerchantOffers and Barter decks, deterministic visits, and barter flows recorded to the ActionLog.

