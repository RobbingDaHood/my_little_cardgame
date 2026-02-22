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
- Token lifecycle & actions log: Tokens must declare lifecycle semantics and every grant/consume/expire/transfer must be recorded in the actions log so runs are auditable and reproducible.
- Single mutator endpoint: All state mutations must be performed via a single POST /action endpoint (the "action" endpoint) which accepts structured action payloads and appends atomically to the ActionLog; other endpoints are read-only.
- All gameplay state mutations must be performed via POST /action. Testing and debugging endpoints under /tests/* are exceptions and should be documented as temporary testing utilities.

Roadmap steps
--------------
1) Refactor library: unify decks, hands, tokens, and enforce vision constraints
   - Goal: Create a single library crate that is the authoritative implementation of decks, tokens, Library semantics, and the canonical token registry.
   - Description: Extract Deck, Hand, Zone, Token, CardDef, Library and ActionLog types; implement a token registry with lifecycle metadata and a compact actions log API to record lifecycle events.
   - Playable acceptance: Unit tests and property tests for deck/token invariants; an API endpoint GET /library returns canonical card entries; actions log records simple token grant/consume events.
   - Notes: Make the library the only place that mutates authoritative game state; surface a small, well-documented API and enforce "everything is deck/token" at type level.

2) Implement global seeded RNG and deterministic execution primitives
   - Goal: Add a single-game RNG and a deterministic scheduler that all systems use (deck shuffles, encounter generation, affix rolls, combat decisions where non-deterministic choices exist).
   - Description: Provide RNG seeding at game/session creation, utility methods to derive deterministic sub-seeds, and deterministic replay helpers (serialize/deserialize seeds and RNG state). Integrate RNG into the ActionLog to record key random draws.
   - Playable acceptance: Starting a session with a seed and replaying the run reproduces identical outcomes for a seeded test scenario.
   - Notes: Make it trivial to replay a logged run by restoring seed + action sequence.

3) Implement append-only Actions Log endpoint and structured actions API
   - Goal: Provide an append-only actions log endpoint (GET /actions/log) and a structured action API to record every grant/consume/expire/transfer, deck movement, and RNG draw so runs can be reproduced from seed + action list.
   - Description: Implement an append-only, chronologically ordered ActionLog that records action metadata, RNG draws, and lifecycle events; expose GET /actions/log and an internal append API for server components to write atomic entries. Ensure log entries contain sufficient metadata to reconstruct game state when combined with the initial seed.
   - Playable acceptance: API returns chronologically ordered action entries and a replay test reconstructs state from seed + action log.
   - Notes: Make the ActionLog the canonical audit trail; ensure deterministic operations write to the log atomically and include lifecycle metadata (reason, resulting state). Designers may choose whether expired tokens are archived (kept in history) or removed; the ActionLog must record which behaviour is selected for each token type.
    - The append-only ActionLog is the authoritative audit trail for all state changes. Every card movement between zones (Hand, Deck, Discard, Deleted), every token grant/consume/expire, and every random draw are recorded with metadata (reason, amount, timestamp, resulting state) so the game state can be reconstructed from seed + action sequence for validation, testing, and replay.

4) Implement canonical token list and lifecycle enforcement
   - Goal: Add the canonical token definitions (Insight, Renown, Refinement, Stability, Foresight, Momentum, Corruption, etc.) and lifecycle classes from vision.md and ensure the ActionLog records lifecycle events.
   - Description: Implement token types, caps/decay rules, and lifecycle metadata. Ensure actions API records token events with reason, amount, and resulting state.
   - Playable acceptance: Tests assert lifecycle transitions (grant, consume, expire) for at least three token types and actions log entries are produced.
   - Notes: Keep the canonical token list authoritative and extensible via the library crate.
    - Current token registry (scope of Step 4): Health, Dodge, Stamina (basic survival tokens used in current combat).
    - Future token registry (Step 4 onwards): Insight, Renown, Refinement, Stability, Foresight, Momentum, Corruption, Purity, and discipline-specific tokens.
    - Each token type must declare its lifecycle (Permanent, PersistentCounter, FixedDuration, etc.) in the canonical registry.

5) Add Area Decks with encounter removal + replacement and scouting hooks
   - Goal: Introduce AreaDecks that contain encounter cards and support the vision's replace-on-resolve behavior: resolved encounter cards are removed and replaced by freshly generated encounters with affixes; scouting biases replacement generation.
   - Description: Implement AreaDeck, encounter consumption, replacement-generation (base type + affixes), and a simple affix pipeline. Implement binding of encounter decks to the encounter instance (encounter deck, reward deck, modifier pulls) and ensure any entry_cost for attempting an encounter is consumed/locked at start. Expose GET /area/{id}/draw and POST /area/{id}/replace endpoints and ensure all deck-bound draws, entry_cost consumes, and replacement-generation steps are recorded in the ActionLog.
   - Playable acceptance: Drawing and resolving an area encounter removes it from the area deck and immediately creates a replacement entry; scouting-related parameters can bias replacement generation in deterministic tests.
   - Notes: Start with small affix sets and deterministic replacement rules.

6) Refactor combat into the library core (deterministic, logged)

   - Note: CombatAction is a simple card-play struct { is_player, card_index } and CombatSnapshot replaces CombatState in the library-centric design.

   - Goal: Move combat resolution, deterministic start-of-turn draws, turn order, actions, and enemy scripts into the shared library, using the seeded RNG and writing a deterministic actions log.
   - Description: Define CombatState, CombatAction, enemy scripts, and resolve_tick/resolve_turn methods that produce an explicit, replayable combat log. Ensure start-of-turn mechanics (draws, tempo, and turn order) are deterministic and driven by the session RNG. Integrate combat events into the ActionLog so every state change is auditable.
   - Playable acceptance: POST /combat/simulate accepts a CombatState and seed and returns a deterministic combat log that reproduces when replayed.
   - Notes: Keep combat pure-data where possible and surface minimal side-effecting entry points that only write to the action log.

7) Add the simple encounter play loop (pick -> fight -> replace -> scouting)
   - Goal: Support a single-playable encounter loop as described in the vision: pick an encounter, resolve it, perform the post-encounter scouting step, and repeat. 
   - Description: Implement /encounter/start, /encounter/step, /encounter/finish flows that use in-memory session state for now and write all events (including replacement and scouting decisions) to the ActionLog.
        - Remember that the action endpoint is the only endpoint allowed to change state. When the player plays an action (examples: pick an encounter, play a card, etc.), the game evaluates whether that changes any state (for example: move the combat one phase forward, conclude the combat, and go to the post-encounter scouting step, etc.). 
   - Playable acceptance: API user can draw an encounter, resolve combat to conclusion, perform a scouting post-resolution step that biases replacement, and the area deck updates accordingly.
   - Notes: Ensure session can be replayed from seed + action log.
    - - Scouting parameters (preview count, affix bias, pool modifier) are internal mechanics that influence encounter-generation deterministically during the scouting post-encounter step. They are not user-facing API endpoints but are controlled by the player's scouting action choices and token expenditures (Foresight, etc.).

7.5) Unify combat systems and remove old deck types
   - Goal: Unify the two combat implementations (src/combat/ old HTTP-driven Combat/Unit/States and library::combat deterministic CombatSnapshot/CombatAction) so a single authoritative combat system resolves card effects and token lifecycles.
   - Description: Migrate resolve_card_effects to read from the Library and player state consistently, replace CombatState with CombatSnapshot, and adopt CombatAction as a simple struct { is_player, card_index }. After unification, remove legacy Deck, DeckCard, CardState from src/deck/ and player_data.cards, and migrate or remove test endpoints that rely on legacy deck CRUD.
   - Playable acceptance: A single combat API backed by library::combat produces deterministic CombatSnapshots, reconciles card definitions and locations with the Library, and provides a clear migration path for removing legacy deck types.
   - Minimal playable loop: After this step introduce a very simple game loop: pick an encounter, play cards until one side has lost all HP, run a quick scouting phase (Just add the current finished encounter card back into the encounter "deck": Change the library counters -1 on discard +1 on deck (Later we will expand on this setup)), then prepare to pick another encounter.

7.6) Flesh out combat and draw mechanics
   - Goal: Implement basic resource-card draw mechanics and encounter handsize rules to make pacing simple and deterministic.
   - Description: Resource cards are the only way to draw additional cards into hands: playing a resource card triggers draws onto one or more hands and is the primary way players gain cards to their hand. Enemies follow the same principle: certain enemy cards act as resource/draw cards that cause draws for their hands.
   - Encounter handsize & Foresight: The encounter handsize is controlled by the Foresight token (default starting value: 3). When an encounter is chosen it is moved to the discard pile and when the encounter is over cards are drawn until the "area deck" hand reaches the Foresight number of cards (this behavior applies to area/encounter hand management).
   - Enemy play behavior: On each enemy turn the enemy plays a random card from its hand for each of its three decks; playing may trigger draws as described, so enemies will sometimes draw new cards.
   - Deck composition: Ensure starting decks for both players and enemies contain approximately 50% draw/resource cards so games have steady card-flow and pacing.
   - Playable acceptance: A minimal loop exists (pick -> fight -> scouting -> pick) with resource-card driven draws, Foresight-controlled encounter hands, enemy random play, and starting decks containing ~half draw cards.

8) Expand encounter variety (non-combat and hybrid encounters) — gathering first
   - Goal: Add gathering (Mining, Woodcutting, Herbalism) and other encounter types that reuse the cards-and-tokens model and discipline decks, and produce raw materials required for crafting.
   - Description: Implement node-based gathering encounters where discipline decks resolve the node (e.g., Mining uses Mining deck vs IronOre card) and produce raw/refined material tokens. Ensure Discipline Durability and Rations semantics are enforced; failures produce Exhaustion or Durability loss. Record material token grants in the ActionLog so crafting has a provable input history.
   - Playable acceptance: At least one gathering discipline is playable end-to-end, produces material tokens consumed by craft flows, and actions are routed via POST /action.
   - Notes: Ensure node resolution follows the same remove-and-replace lifecycle and that scouting can affect yields.

9) Add basic crafting as discipline encounters and respect Library semantics
   - Goal: Implement crafting as discipline-specific encounters (Fabrication, Provisioning, etc.) that use discipline decks, consume materials, and create Library card copies when finalized.
   - Description: Model craft encounters with discipline decks that supply action cards; deterministic craft resolution produces Library card copies with rolled affixes and logs all material/token spends. Enforce affix constraints (affix types are fixed once attached by the Modifier pipeline) and make Refinement/Stability tokens affect numeric roll bias/variance.
   - Playable acceptance: A craft endpoint resolves a craft encounter, produces a Library card copy (visible via GET /library), records costs/spends in the ActionLog, and demonstrates cost-scaling with affix count/quality; crafted cards are never directly inserted into player decks.
   - Notes: Start with a single discipline (e.g., Fabrication) and one recipe to prove the flow; ensure crafting is the primary economy sink and costs scale with affix count/quality and Variant-Choice/Affix-Picks usage.

10) Add Research/Learning and Modifier-Deck pipeline
   - Goal: Implement Research/Learning as first-class encounters that produce Insight, Variant-Choice, and Affix-Picks and generate Library variants via a Modifier Deck workflow.
   - Description: Add a ResearchDeck and ModifierDeck types; support presenting X research candidates (driven by tokens like "max research choice"), selecting one to start, and resolving research by drawing modifier cards and attaching up to Y affixes (Y from Affix-Picks). Random draws and affix numeric rolls use the global seed and all steps are recorded in the ActionLog. Completed variants are added to the Library (never directly to player decks).
   - Playable acceptance: A Research flow endpoint presents candidates, accepts a selection, resolves modifier draws deterministically using the session seed, produces a Library variant, and logs every decision/draw.
   - Notes: Start with a small ResearchDeck and ModifierDeck; ensure variant generation and replay are deterministic and auditable. ModifierDeck entries come primarily from Milestones/Challenge rewards and unique modifier acquisition; implement rules to replenish ResearchDeck candidates after resolution so research remains a continuous pipeline.

11) Add post-encounter scouting choices (vision-driven)
   - Goal: Present scouting choices as a post-resolution step that influence replacement-generation parameters (preview counts, affix biases, candidate pools) and grant Foresight/related tokens.
   - Description: Implement ScoutChoice objects and a deterministic application that updates replacement parameters for the specific area deck. Record scouting decisions and effects in the ActionLog.
   - Playable acceptance: After an encounter, API returns scouting choices; making a choice updates the replacement-generation seed/parameters and is reflected in the next replacement card deterministically.
   - Notes: Keep initial choices small and data-driven (e.g., +1 Foresight, increase affix-pool size).
    - Up to this point then all encounters just added the same encounter back into the area deck: no changes. 

12) Implement Trading and Merchants (MerchantOffers + Barter workflow)
   - Goal: Model merchants as decks (MerchantOffers, Barter) and deterministic merchant interactions that mirror vision.md's barter mechanics.
   - Description: Implement MerchantOffers deck and Barter deck support. Merchant interactions deterministically draw a MerchantOfferPool, present offers, then draw MerchantBargainDraw barter cards and allow choosing up to MerchantLeverage to modify the selected offer. Barter cards can change offered_token, requested_token, rate, fee, or attach conditions. All draws, choices, and resulting token transfers are recorded in the ActionLog so merchant interactions are reproducible.
   - Playable acceptance: A /merchant/{id}/visit endpoint returns deterministic offers derived from the session seed; applying barter choices updates player tokens and is recorded.
   - Notes: Start with a single merchant and simple barter cards; expand to dynamic merchant inventory later.

13) Finalize edge cases for a repeatable loop and concurrency
   - Goal: Make the loop robust: reshuffle/renew rules, player death and recovery, encounter exhaustion and replacement guarantees, and multi-session concurrency safety.
   - Description: Add tests for deck exhaustion/reshuffle, rules for encounter removal+replacement when decks empty, and concurrency controls for per-session AreaDeck mutations.
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

How to use this roadmap
-----------------------
- Use steps as milestones for sprints or PRs. Each PR should implement a step's minimum playable acceptance criteria and include tests that reproduce behaviour from a seed.
- After completing each step, add at least one integration test that exercises the end-to-end playable loop for that milestone and verifies action-log replay.
- Iterate on balance and UX only after mechanics are stable and auditable.

Appendix: Minimum ticket examples for the first 8 steps
-----------------------------------------------------
- Refactor library: Extract Deck/Hand/Zone/Token/Library into lib crate, add unit tests for move/draw/reshuffle, and implement a canonical token registry.
- RNG: Add seeded RNG, deterministic derive API, and replay helper for restoring runs.
- Token lifecycles: Implement Insight/Foresight/Renown/Refinement/Stability and action-log recording for lifecycle events.
- Area deck: Create AreaDeck with seed data, implement draw/resolve/replace and affix replacement pipeline.
- Combat refactor: Implement CombatState and resolve_tick using seeded RNG and output deterministic logs recorded to the ActionLog.
- Encounter loop: Implement /encounter/start, /encounter/step, /encounter/finish endpoints, include replacement and scouting as part of the lifecycle.
- Research: Implement ResearchDeck + ModifierDeck pipeline and deterministic variant generation recorded to the ActionLog.
- Merchants: Implement MerchantOffers and Barter decks, deterministic visits, and barter flows recorded to the ActionLog.

