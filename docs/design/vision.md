# Vision: My Little Cardgame

This document describes the high-level gameplay vision for the project, with a focus on core mechanics and the role of decks and tokens.

## High-level principles

- Turn-based gameplay: All interactions are resolved in discrete turns. Players (and enemies/encounters) take actions in a clear turn order, allowing deterministic resolution and meaningful choices.
- Everything is a deck or a token: Cards, encounters, recipes, resources, merchants, and area encounters are modelled as decks (and individual items as tokens/cards drawn from those decks).
- Fully reproducible: The game is initialized with a single explicit random seed at new-game start; all shuffles and random choices are derived from that seed so the entire game can be reproduced or replayed exactly.
- Intentional minimal canonical data: the authoritative game state deliberately omits titles, verbose descriptions, and flavor text; only structural names, types, tokens, and essential parameters are stored. Presentation and flavour are delegated to clients and separate design notes.

## Architecture and module layout

The codebase is organized into the following main modules:

- `src/library/` — core domain module containing:
  - `types.rs` — core data types (LibraryCard, CardKind, CardEffectKind, CombatantDef, CombatState, ActionPayload, CardLocation, etc.)
  - `action_log.rs` — append-only player action log
  - `game_state.rs` — top-level GameState managing Library, tokens, combat, and encounter phase
  - `endpoints.rs` — HTTP route handlers for gameplay and library queries
- `src/combat/` — combat state endpoints (delegates to GameState methods)
- `src/action/` — player action handling and POST /action request processing (PlayerActions enum manages encounter phase transitions)
- `src/player_data.rs` — RandomGeneratorWrapper: seeded RNG wrapper for deterministic random operations
- `src/main.rs` — binary entry that mounts Rocket routes and serves OpenAPI/Swagger UI

## Core gameplay elements

Deck types (examples):
- Player decks: Attack, Defence, Resource, Mining, Fabrication, Provisioning, Woodcutting, Barter (for trading)
- Encounter decks: Enemy, Trap, Puzzle
- Treasure decks: Ore, Lumber, Herbs, Loot, MerchantOffers
- Recipe deck: Recipes that can be drawn/selected to craft items
- Area deck: Represents the player's current location. Encounter cards live in the Library and use the same CardCounts (library/deck/hand/discard) and CardLocation tracking as all other card types. There is no separate AreaDeck struct; encounter card management uses Library helper methods (encounter_hand, encounter_contains, encounter_draw_to_hand). The hand represents visible/pickable encounters and its size is controlled by the Foresight token (default 3). Encounter cards are accessed via `/library/cards?location=Hand&card_kind=Encounter`.
- Library: a canonical collection of all cards accessible via the library endpoint; any card present in the library can be added to a deck provided the deck is of the appropriate type (deck-type constraints apply)

  - Library implementation notes: Card definitions use `effect_ids: Vec<usize>` to reference their CardEffect entries in the Library by index. Effect data lives on `PlayerCardEffect` and `EnemyCardEffect` library entries which hold `kind: CardEffectKind` directly. The Library is implemented as Library { cards: Vec<LibraryCard> } where each LibraryCard index serves as the canonical card id; new cards must always be appended to the end to preserve stable IDs. CardKind (enum, serde tag `"card_kind"`) replaces ad-hoc string-based card types (Attack{effects}, Defence{effects}, Resource{effects}, Encounter{kind: EncounterKind}, PlayerCardEffect{kind: CardEffectKind}, EnemyCardEffect{kind: CardEffectKind}). EncounterKind is an enum starting with Combat { combatant_def }; enemy card definitions are embedded inline within Encounter definitions rather than as separate Library references.
  - The Library contains two CardEffect "decks": PlayerCardEffect cards (used during research to generate new player cards) and EnemyCardEffect cards (used during the post-encounter scouting phase to generate new encounters). Every card effect on action/enemy cards references a CardEffect deck entry via `effect_ids: Vec<usize>`. API responses show full effect values, not references. All card effects on player cards must reference a valid PlayerCardEffect entry; all effects on enemy cards must reference a valid EnemyCardEffect entry (validated at Library initialization).


The Player's decks (Attack, Defence, Resource etc.) are fixed and initialized at game start. Only the Library manages the canonical card definitions and the internal deck representation. The API does not expose deck-creation or deck-deletion endpoints; player deck composition is managed only through adding Library cards to decks via the deck-management flow.

Tokens and state:
- Tokens represent persistent or semi-persistent objects like equipped items, active status effects, or resource counters.
- Moving a card between decks (Hand, Deck, Discard, Deleted) represents the lifecycle of that object or effect.

### Canonical Data Omits Flavor

The canonical Library stores only structural identifiers (card IDs, types, tokens, numeric parameters). All user-facing names, descriptions, and flavor text are delegated to client presentation layers. The API responses use only ID-based references; naming/presentation is the client's responsibility based on a separate design specification. This ensures the game state remains minimal, reproducible, and suitable for replay and analysis. It also leaves a lot to the imagination of the player. 

## Current combat setup

### Implementation updates (2026-02-23)
- Token identifiers are a closed `TokenType` enum: Health, MaxHealth, Shield, Stamina, Dodge, Mana, Insight, Renown, Refinement, Stability, Foresight, Momentum, Corruption, Exhaustion, MiningDurability, Ore, OreHealth. A `Token` is a struct containing `token_type: TokenType` and `lifecycle: TokenLifecycle`. Token maps use `Token` as the key and `i64` as the value. PersistentCounter is the default lifecycle for all token types, but any token type can use any lifecycle — for example Dodge uses FixedTypeDuration { duration: 1, phases: [Defending] }. Token maps serialize as compact JSON objects (e.g., `{"Health": 20}`); the old array-of-entries format is still accepted for backward-compatible deserialization.
- The ScopedToEncounter lifecycle was replaced by FixedTypeDuration { phases, duration }; references to ScopedToEncounter were removed.
- Encounter state lives in GameState (current_encounter: Option<EncounterState>, encounter_phase: EncounterPhase) and src/combat/ endpoints delegate to GameState methods. EncounterState is an enum with variants Combat(CombatEncounterState) and Mining(MiningEncounterState), each containing encounter-type-specific state. CombatEncounterState tracks enemy tokens directly (enemy_tokens field) along with mutable copies of enemy decks and combat metadata; player tokens live on GameState.token_balances, not inside the encounter state. Turn control is implicit: the player always acts first, then the system auto-resolves enemy play and advances combat phase within the same action.
- Encounter outcome is tracked via an `EncounterOutcome` enum on each encounter state with variants: Undecided, PlayerWon, PlayerLost. GameState maintains `encounter_results: Vec<EncounterOutcome>` — each completed encounter pushes its outcome to this vector. The `/encounter/results` endpoint returns the full history. Encounter completion is determined solely by `outcome != EncounterOutcome::Undecided` (there is no separate `is_finished` field).
- Dodge absorption: damage consumes Dodge tokens first before Health is reduced.
- Drawing additional cards is modelled via the `CardEffectKind::DrawCards { attack, defence, resource }` variant of the `CardEffectKind` enum. `CardEffectKind` has two variants: `ChangeTokens { target, token_type, amount }` for token manipulation, and `DrawCards { attack: u32, defence: u32, resource: u32 }` for per-deck-type card draw. Resource cards trigger draws via their effects list, just like attack and defence cards trigger damage/shield via ChangeTokens effects. Starting decks draw 1 attack, 1 defence, 2 resource cards per resource play (4 total) for steady pacing. DrawCards is not a token type — it is purely a card effect subtype. Both player and enemy draws happen per deck type with discard recycling per type.
- CardEffectKind extensibility: new card effect subtypes (e.g., conditional effects, area-of-effect, combo triggers) should be added as new `CardEffectKind` variants rather than overloading `ChangeTokens` with special token types. This keeps each effect kind self-describing and ensures exhaustive match coverage in Rust.
- Enemy decks use `DeckCounts { deck, hand, discard }` (a generic struct shared by all encounter-internal deck tracking — enemy decks, ore decks, and future encounter card pools) to track card locations. At combat start, enemy hands are shuffled (all cards move to deck, then random cards drawn to restore hand size using the seeded RNG). Enemies play from hand only; played cards go to discard. When a deck is empty and a draw is needed, the discard pile is recycled into the deck. Resource cards draw their DrawCards amounts (via CardEffectKind::DrawCards { attack, defence, resource }) of cards for each of the three enemy deck types, providing the only replenishment mechanism for all enemy decks.
- After a player plays a card via EncounterPlayCard, the system automatically resolves the enemy's play and advances the combat phase. There are no separate endpoints for enemy play or phase advancement. The exact auto-advance sequence is:
  1. Player plays card → resolve player card effects
  2. Resolve enemy play (one random card matching current CombatPhase from enemy hand)
  3. Advance combat phase (Defending → Attacking → Resourcing → Defending)
  4. Check combat end conditions (either side HP ≤ 0)
- CardKind::Encounter { kind: EncounterKind } replaces the old CombatEncounter variant. EncounterKind is an enum with variants Combat { combatant_def } and Mining { mining_def }, with future non-combat encounter types to follow (Woodcutting, Herbalism, etc.). CardKind::Mining { mining_effect: MiningCardEffect } is a separate card kind for mining action cards with inline effects (ore_damage: i64, durability_prevent: i64).
- Players cannot abandon combat encounters once started; combat continues until one side is defeated (HP reaches 0). Non-combat encounters (e.g., Mining) may be aborted via the EncounterAbort action, which marks the encounter as PlayerLost, grants no rewards, applies no penalties, and transitions to Scouting phase. EncounterApplyScouting is the action that transitions from post-encounter Scouting phase back to NoEncounter.
- Scouting after loss: after combat ends (whether the player wins or loses), the game transitions to the Scouting phase. The player can apply scouting regardless of combat outcome. This is intentional — scouting is a post-encounter lifecycle step, not a victory reward.
- Health initialization: player Health is set to 20 when picking an encounter only if current Health is 0. Health persists across encounters within a game; it is not reset between encounters.
- All amounts are positive: attacks have a positive number even though they remove health points, allowing unsigned integers to be used where possible.
- Five player actions exist: NewGame { seed: Option<u64> }, EncounterPickEncounter { card_id }, EncounterPlayCard { card_id }, EncounterApplyScouting, EncounterAbort. EncounterAbort allows aborting non-combat encounters (returns 400 for combat). All other previously documented actions have been removed.
- NewGame { seed: Option<u64> } initializes a fresh game. If no seed is provided, a random one is generated. The old /player/seed endpoint and SetSeed action have been removed.
- The action log records only player actions (NewGame, EncounterPickEncounter, EncounterPlayCard, EncounterApplyScouting, EncounterAbort). Internal operations (token grants, consumes, card movements) are deterministic consequences of player actions and the seed, so they do not need logging for reproducibility. ActionEntry contains only `seq: usize` (index in the log) and `payload: ActionPayload`. The `/actions/log` endpoint returns chronologically ordered entries.
- CardLocation enum: `Library`, `Deck`, `Hand`, `Discard`. Used as a query filter on `/library/cards?location=Hand&card_kind=Encounter`. All card types use the same location tracking system via CardCounts. The `/library/cards` endpoint returns `LibraryCardWithId` (includes card ID/index) with optional `?location=` and `?card_kind=` filters.
- Card draws are random (seeded): `draw_player_cards_of_kind` accepts an RNG parameter and picks a random card from the drawable pool, not the first card sequentially.
- CombatPhase::allowed_card_kind() returns a type-safe predicate `fn(&CardKind) -> bool` rather than a string comparison.
- CombatantDef.initial_tokens uses `HashMap<Token, u64>` (unsigned); runtime token_balances uses `i64` for signed arithmetic during damage calculations.
- Token definitions live in the `TokenType` enum and `Token` struct (token_type + lifecycle). There is no separate TokenRegistry; tokens are created directly from TokenType via constructors like `Token::persistent(token_type)` and `Token::dodge()`. GameState.token_balances is the sole source of truth for token state.

## Current combat setup

Combat is modelled as a deterministic, turn-based exchange between decks:
1. Encounter selection: The player picks an encounter card from the encounter hand (visible Library encounter cards). The handler dispatches to the appropriate encounter start method based on EncounterKind (Combat or Mining).
2. Setup: Enemy card exposes stats (HP, Attack, Defence, special effects) via its CombatantDef. Player tokens (Health, Shield, etc.) live on GameState, not inside the combat snapshot.
3. Player turn: The player plays a card via EncounterPlayCard. The system resolves the card's effects (attack deals damage, defence grants shield, resource triggers draws via DrawCards effects).
4. Auto-advance: After the player's card, the system automatically resolves the enemy's play (one random card matching the current CombatPhase) and advances the combat phase (Defending → Attacking → Resourcing → Defending).
5. Resolution: Cards moved to Discard or Deleted. Repeat until one side is defeated (HP reaches 0). Encounter outcome is tracked via EncounterOutcome enum (Undecided, PlayerWon, PlayerLost) on the encounter state. Each encounter state has a mandatory `encounter_card_id: usize` tracking which library card spawned it.

Combat is fully reproducible by recording the game's single initial seed and the player's action log (which card was played each turn).

  - Encounter phases: the `EncounterPhase` enum defines the encounter state machine with the following variants:
    - `NoEncounter` — no encounter is active; the player may pick an encounter from the encounter hand
    - `InEncounter` — an encounter is in progress (combat, mining, or any future encounter type). The specific encounter type is determined by the `EncounterState` enum variant, not by the phase.
    - `Scouting` — post-encounter phase where the player applies scouting before returning to NoEncounter
  - Note: `EncounterPhase` is stored directly on `GameState.encounter_phase`.

  - HP as tokens: Hit points are modelled as tokens (e.g., health and max_health in active_tokens) rather than dedicated fields, following the 'everything is a token' principle.

  - EncounterState pattern: `EncounterState` is an enum dispatching encounter-type-specific logic:
    ```rust
    pub enum EncounterState {
        Combat(CombatEncounterState),
        Mining(MiningEncounterState),
        Herbalism(HerbalismEncounterState),
        Woodcutting(WoodcuttingEncounterState),
        Fishing(FishingEncounterState),
    }
    ```
    Each encounter type has its own state struct because mechanics differ fundamentally. Combat uses 3 decks + phases (Defending/Attacking/Resourcing); mining uses a single deck with no phases; herbalism uses card-characteristic matching with no enemy draws; woodcutting has no enemy deck and uses pattern evaluation; fishing uses card-subtraction with valid-range targeting. The `EncounterState` enum provides helpers like `is_finished()` (checks `outcome != Undecided`) and `encounter_card_id()`.

  - Mining encounter mechanics (implemented):
    - Single-deck resolution: Player Mining deck contains cards with `ore_damage` (damage to ore node) and `durability_prevent` (reduces incoming durability damage) as a tradeoff. Ore node has an OreDeck with cards dealing 0-3 durability damage (distribution skewed low: ~30% zero, ~40% one, ~20% two, ~10% three).
    - Turn flow: Player plays mining card → ore_damage reduces OreHealth token → durability_prevent is computed inline → ore plays random card from ore deck → effective durability damage = raw - prevent → both sides draw → check end conditions.
    - Win: OreHealth ≤ 0 → grant reward tokens (e.g., Ore: 10). Rewards use `HashMap<Token, i64>` keys (full Token with lifecycle, not just TokenType).
    - Lose: MiningDurability ≤ 0 → encounter ends as PlayerLost with no additional penalties. MiningDurability loss across encounters IS the implicit penalty.
    - No phases: Unlike combat's Defending → Attacking → Resourcing cycle, mining resolves one action per turn with no phase rotation.
    - Ore health tracked via `ore_tokens: HashMap<Token, i64>` with OreHealth token (encounter-scoped). Serialized JSON: `"ore_tokens": {"OreHealth": 15}`.
    - MiningDurability: Initialized to 100 in `GameState::new()` at game start. Persists across all mining encounters and decreases over time. NOT re-initialized per encounter. High initial value (100) is a placeholder pending repair mechanics.
    - Endpoints: `/encounter` and `/encounter/results` serve all encounter types. Response JSON includes `encounter_state_type` discriminator field (`"Combat"`, `"Mining"`, `"Herbalism"`, `"Woodcutting"`, or `"Fishing"`).


## Areas as decks (future vision)

- Visiting an area is modelled by opening an Area deck: each card is an encounter or a treasure.
- Example: Mine area has an `Iron Ore` treasure deck and an `Ore Encounter` deck. When you visit a mining node, a card is drawn from the Iron Ore deck to determine the type/quality of ore and any modifiers.
- At any given time, the player has one active area deck representing their current location. Moving to a new area replaces the current area deck. This simplifies the state model.

Example - Iron Ore node:
- Player visits a mine node: a card is drawn from the ore deck.
- The drawn card might be `SmallIronVein`, `EncrustedIron`, or `ElementalCore` with varying difficulty and loot.
- Some treasure spots explicitly allow alternate interactions such as "learning" or scouting-related actions: learning can grant recipes, lore, or crafting shortcuts; scouting is not a standalone encounter but a post-resolution step that changes the area deck and hand. After any encounter resolves (win or loss) the resolved card is removed from the Area deck and immediately replaced by a new encounter of the same base type with affix modifiers; scouting controls how those replacement encounters are generated (see Scouting mechanics below).
- A special "combat" starts where the player's `Mining` deck is used to interact with the ore card: mining cards represent mining tools/techniques, resource cards provide stamina/endurance, and failure/success is resolved as if the ore were an enemy with HP and resistances.
- Success yields one or more specific treasure cards (iron ingot tokens, rare gems), moved into the player's Loot/Inventory deck.

Other gathering professions follow the same model: lumber nodes draw from a `Lumber` deck and use the `Woodcutting` deck to resolve the encounter; herbalism uses `Herbalism` decks; each profession uses its own action/crafting deck to "defeat" the resource card.

## Crafting

- Crafting is resolved as a discipline-specific encounter: choose a recipe (from the Recipe deck or Library) and resolve the craft using the relevant discipline deck (Fabrication/Weaponcraft for physical items and tools, Provisioning for consumables and reagents, Woodcutting for node-sourced inputs, etc.). The discipline deck supplies action cards and choices that determine success, quality, and side-effects.

- Library-only placement: crafted card copies are always placed into the Library when completed; crafting never inserts cards directly into player decks. Players add Library cards into decks later via normal deck-management flows and subject to deck-type constraints.

- Crafting cost and scaling: every card has an intrinsic crafting cost that scales with the number and rarity of attached affixes and with how favorable the affix numeric rolls were (higher-quality variants cost more to reproduce). Consumable resources are drawn from inventory decks to pay base costs.

- Affix constraints: affix types are fixed once created by the Modifier Deck process; crafting cannot change affix types or categories. Crafting can, however, reroll or refine numeric values of affixes (at resource/token cost and risk) and improve or reduce variance using dedicated tokens.

- Tokens and trade-offs: Renown and Insight are earned via risky or showy plays; they are spendable across systems (merchants, research boosts) and may be used in specialized crafting flows as designer intent requires. Refinement and Stability tokens are the primary currencies for improving craft rolls (Refinement raises expected values, Stability reduces variance). There are also discipline cards and cost-reduction cards that can lower resource/token costs when played during the craft encounter.

- Craft encounter choices: while resolving a craft the player may spend Refinement/Stability to reroll or bias numeric values, push for higher quality (increasing cost), or accept penalties (which can grant discipline-specific tokens or short-term benefits). Failures can consume extra resources, add Exhaustion, or introduce Corruption as designed trade-offs.

- Finalization and reproducibility: a successful craft creates a new Library card copy with its rolled values and affixes; all draws and random rolls are deterministically derived from the game's single initial seed and recorded so outcomes are reproducible. Crafted cards join the Library catalog and are available for future deck composition or as targets of further research/crafting cycles.

- Design note: keeping crafting focused on value refinement (not affix-type mutation) creates a clear, discipline-centred refinement loop that uses the same cards-and-tokens language as other systems while preserving predictable upgrade paths and reproducibility.

### Crafting overview: disciplines, material flows, and token mapping

This overview maps gathering, refining, and crafting disciplines together and enumerates the authoritative token list with which disciplines generate and consume them. The end goal is crafting cards; cards may require raw materials, refined materials, and a variety of tokens (currency, modifiers, progression tokens) supplied by different disciplines and systems.

High-level flow

- Gathering disciplines (Mining, Woodcutting, Herbalism, Fishing, etc.) generate raw material tokens (Ore, Lumber, Herbs, Fish, etc.).
- Refining crafting disciplines (Fabrication, Provisioning, etc.) consume raw materials and produce refined material tokens (IronBar, Plank, Reagent, Tincture) that other crafting disciplines can use as inputs to craft final card definitions.
- Crafting encounters consume refined materials, discipline-specific action cards, and craft tokens (Refinement/Stability) to produce new Library card copies or modified variants.
- Research/Learning, Milestones, and scouting-related systems supply higher-level tokens (Variant-Choice, Affix-Picks, Insight, Foresight) that influence variant generation, choice breadth, and unlock paths.

Canonical token list, generators, and uses

- CurrentResearch (structured): generated/owned by Research/Learning flows; payload {research_card_id, progress, started_at, metadata}. Use: authoritative pointer to the active research project and its progress; not spendable.

- Insight (per-discipline): primarily generated by Learning/Research encounters and some gathering outcomes. Spendable: yes. Use: start/accelerate research, pay entry for Milestone attempts, expose library alternatives.

- Renown (per-discipline): generated by showy or taunting plays across combat/gathering/crafting and some milestone rewards. Spendable: yes. Use: spend at merchants or for special reputation-based offers and discounts.

- Variant-Choice (numeric): generated rarely via Milestone rewards or special progression sources. Spendable: yes. Use: controls how many modifier cards are drawn when generating research replacement variants.

- Affix-Picks (numeric): generated via deep progression (milestones, rare rewards). Spendable: yes. Use: controls how many modifiers may be attached during variant creation.

- Refinement (numeric): generated by milestone rewards, some crafting outcomes, and possibly purchasable via merchants or special encounters. Spendable: yes. Use: bias or reroll numeric affix values during Craft encounters to increase expected quality.

- Stability (numeric): generated similarly to Refinement (progression rewards, milestones). Spendable: yes. Use: reduce variance on affix rolls during Craft encounters.

- Momentum (combat token): generated by chaining offensive plays or foregoing defence. Spendable: yes. Use: trigger follow-up/combo effects in combat and some discipline synergies.

- Foresight (scouting token): earned via scouting actions and reconnaissance-related rewards. Spendable: yes. Use: Decides the max hand size of the area "deck": After an encounter have been resolved then draw area cards until Foresight amount of cards were drawn. 

- Scouting candidate pool: earned via scouting actions and reconnaissance-related rewards. Spendable: yes. Use: During the scouting post-encounter step, then this decides how many "affixes" are drawn to "build" the next encounter. 

- Scouting pick count: earned via scouting actions and reconnaissance-related rewards. Spendable: yes. Use: During the scouting post-encounter step, then this decides how many "affixes" that maximum can be choosen to "build" the next encounter. 

- Corruption / Purity (moral tokens): Corruption is generated by forbidden or tainted actions and is reduced or transformed via specific purge mechanics or Purity spends; Purity is earned by restraint and quests and is spendable to purge Corruption or unlock purity-locked content. Spendable: Purity yes; Corruption not a normal currency but can be altered by specified actions. Use: Corruption modifies world responses and content gating; Purity can be spent to purge Corruption.

- Exhaustion (negative token): generated by overexertion (failed or costly actions across disciplines). Use: applied as a persistent penalty (reduced hand size, increased costs) until recovered.

- MiningDurability, WoodcuttingDurability, HerbalismDurability, FishingDurability (discipline HP pools): discipline-specific "hp/life" pools used during gathering encounters. Each gathering profession has its own durability pool (named `{Discipline}Durability`). Initialized at game start (e.g., MiningDurability: 100) and persists across encounters. Decreased by encounter actions; restored by repairs, rest, or specific cards. Spendable: no. Use: models the condition and operational capacity of a discipline during encounters and enforces maintenance/repair flows. When a discipline's durability reaches 0 during an encounter, the encounter ends as PlayerLost with no additional penalties (durability loss IS the penalty).

- Thesis (research progression token): generated by completing Learning/Research encounters, milestone rewards, and special events. Spendable: yes. Use: advance or unlock research projects, pay for special experiment options, or enable limited Variant-Choice/Affix-Picks bonuses during research.

- Key tokens / Special-purpose tokens: generated by Milestone chains and unique challenges. Spendable: yes (on specified unlocks). Use: carry metadata (key_id) and unlock elite or special challenges; often singular/rare.

- Resource counters (raw/refined materials):
  - Ore / IronOre / IronBar: generated by Mining; used by Fabrication/Weaponcraft to produce metal components and final weapon/armor cards.
  - Lumber / Plank: generated by Woodcutting; used by Fabrication, Bowcraft, and some crafting recipes.
  - Herbs / Reagent / Tincture: generated by Herbalism; used by Provisioning and consumable crafting.
  - Rations: primarily produced by Provisioning (preserved or prepared food and reagents) and sometimes found in Loot; Rations are a general-purpose boost token: individual cards across many disciplines may consume Rations to gain targeted benefits (combat stamina/boosts, extraction boosts, crafting quality pushes). Generated by: Provisioning, occasional loot; Consumed by: Combat, and by discipline cards or crafting actions that explicitly require them. Spendable: yes.
  - Loot / MerchantOffers: generated by Area encounters and merchants; used by players as tradeable items or inputs to specific recipes.

- Cost-reduction cards/effects (not tokens): discipline cards or Library effects that reduce resource or token costs for specific encounters or crafts.

Notes: the token list above is canonical for the project; prefer this section as the authoritative reference and avoid duplicating token definitions elsewhere.

## Token lifecycles

Tokens explicitly declare their lifecycle semantics so designers, clients, and the actions log can treat them consistently. Lifecycle classes include:

- Permanent: tokens that persist until explicitly spent or consumed (examples: Key tokens, Library card counts).
- Persistent counters: numeric tokens that persist across sessions but are subject to caps, decay, or refresh rules (examples: Renown, Insight).
- Fixed-duration (X encounters): tokens that expire after N encounters of any type (useful for short buffs or timed boosts).
- Fixed-type-duration (X encounters of a specific type): tokens that expire after N encounters of a specified type (for example, 3 Craft encounters). Implementation note: FixedTypeDuration lifecycles are phase-aware and track which EncounterPhase values they count down during (phases: Vec<EncounterPhase>).

- Until-next-action: tokens that persist until the next player action of a specified type occurs (for example, "until next Research completion").
- Single-use / one-shot: consumed on first applicable use and then removed.
- Conditional: persist until a condition is met (for example MiningDurability hitting 0, Corruption crossing a threshold, or a specific external event).

Rules and implementation notes:
- Every token type must document its lifecycle class, generation rules, caps/decay semantics, authoritative spend paths, and whether it carries structured payload data. Lifecycle is declared solely on the `Token` struct (token_type + lifecycle), not on card effects. Card effects reference a `TokenType` and the lifecycle comes from the token definition. PersistentCounter is the default lifecycle; any token type can use any lifecycle (e.g. Dodge uses FixedTypeDuration { duration: 1, phases: [Defending] }). Token constructors Token::persistent(token_type) and Token::dodge() enforce these defaults. Token definitions live in the `TokenType` enum — there is no separate token registry data structure.
- The actions log records only player actions for reproducibility. Internal token operations (grant, consume, expire) are deterministic consequences of player actions and the seed. The action log combined with the initial seed is sufficient to reproduce all token state transitions.
- Designers may choose whether expired tokens are archived (kept in history) or removed.

Discipline → primary tokens/materials produced (summary)

- Mining: produces Ore, IronOre, occasional Gems; yields raw materials for Fabrication and Refining.
- Woodcutting: produces Lumber and Planks used by Fabrication and tool crafting.
- Herbalism / Botanical: produces Herbs and basic Reagents used by Provisioning.
- Fishing / Foraging: produces Fish and Foodstuffs used by Provisioning or as trade resources.
- Fabrication / Weaponcraft: consumes Ore/IronBar + Refinement/Stability to produce weapon/armor Library entries and refined physical components.
- Fabrication / Bowcraft: consumes Lumber/Plank to produce tools, handles, and refined wooden components.
- Provisioning: consumes Herbs/Reagents to produce Reagent/Tincture tokens and consumable card definitions.
- Provisioning: consumes Ingredients; produces Ration tokens, consumables, or buff cards.
- Research / Learning: generates Insight, CurrentResearch, and occasionally Variant-Choice / Affix-Picks via milestone rewards; produces new recipes/variants.
- Scouting / Recon (system): generates Foresight and other reconnaissance benefits, and can affect resource yields; scouting is a post-resolution/area-update subsystem applied as part of an encounter's lifecycle rather than a standalone encounter type. Scouting preview count = 1 + Foresight token count. Additional scouting parameters (pool modifier) may be derived from other tokens.

- Milestones / Challenge systems: primary source for Variant-Choice, Affix-Picks, rare Refinement/Stability, and Key tokens.

Design implications and notes

- Encourage clear pipelines: ensure that raw materials have meaningful refinement paths so disciplines interlock (e.g., Mining → Fabrication → Weaponcraft).
- Keep tokens scoped: per-discipline tokens (Insight, Renown) should be earned in context and have focused uses to avoid dilution of systems.
- Make refinement meaningful: refined materials should be strictly required for higher-tier cards to create demand across disciplines. Crafting new Library cards is the primary economy sink: the design intent is for crafting to consume or lock up resources and tokens (raw materials, refined materials, Refinement/Stability, Rations, and other progression tokens) so that progression and collection are the ultimate use for player resources.
- Preserve reproducibility: all material transforms, token spends, and variant generation should be seeded and recorded to allow replay and auditing.

(End of appended Crafting overview section.)

## Balancing

Layered balancing approach:

- Resource sinks: ensure crafting new Library cards is the dominant resource sink; require scaled inputs (materials + tokens) that grow superlinearly with affix count and quality.
- Token caps and decay: use caps, soft caps, and decay mechanics to prevent runaway accumulation; apply diminishing returns to token utility at high quantities.
- Rarity and availability tuning: tune drop tables and modifier rarities so progression requires deliberate investment; make certain rare tokens gated behind milestones or unique challenges.
- Cost-scaling: scale crafting costs with affix count, affix quality, and Variant-Choice/Affix-Picks usage so stronger outputs require disproportionate inputs.

Tuning pipeline and instrumentation:

- Instrument metrics: capture resource inflows/outflows, sink rates, median playtime-to-milestone, and token velocity.
- Monte‑Carlo / seeded simulations: run large-scale simulations using the seeded RNG to find pathological economies, measure expected yields, and validate design invariants.
- Regression checks and assertions: automated tests should assert invariants (e.g., average resource lifetime, expected craft throughput per N operations).

Operational controls & feedback:

- Designer knobs: surface parameters like drop rates, Variant-Choice/Affix-Picks frequency, Refinement/Stability supply, and cost multipliers for live tuning.
- A/B and sandbox playtests: perform controlled experiments with telemetry to narrow down balance changes before wide rollout.
- Targeted sinks and throttles: introduce repair/maintenance costs, upkeep, or one-time unlock expenses when telemetry shows runaway accumulation.

(End of balancing section.)

Reward scaling and token economy:
- Degree of success matters: crafting and gathering are not binary win/lose outcomes. The better the player performs in the crafting/gathering encounter (measured by success margin, combo chains, resource efficiency, or other metrics), the greater the rewards — higher-quality loot cards, larger resource yields, or improved token multipliers.
- Token scaling: tokens (resource counters like Wood, Iron Bar, Rations, etc.) are intended to start in the hundreds so that multiplier effects scale smoothly and are not too drastic early in play; this allows meaningful progression while keeping early-game balance manageable.

## Distinct encounter playstyles

It is important that every broad encounter type (combat, gathering, crafting) and each subtype (herbalism, provisioning, learning, etc.) play differently so the player feels varied challenges rather than the same mechanic with renamed cards. Scouting is a cross-cutting reconnaissance system applied as the post-resolution step of an encounter lifecycle, and therefore its concerns (area replacement-generation) are orthogonal to per-discipline playstyles. All differences are expressed using only cards and tokens, but the card types, token economies, success conditions, and meaningful choices should differ.

Examples of how they can differ while staying pure cards-and-tokens:

- Combat (current model): fights are adversarial and reactive. Enemy cards expose HP, attack patterns, and scripted actions; player Attack/Defence/Resource cards are spent to damage, block, or set up combos. Success is measured by reducing enemy HP or meeting encounter objectives; tempo, hand management, and timing matter most.

- Herbalism (gathering subtype): more about knowledge, precision, and sequencing. Herb node cards declare plant characteristics (fragility, potency, required extraction method). Player uses Knowledge and Tool cards (e.g., Botany, Microscope, Distillation) plus Reagent tokens to extract compounds. Success depends on matching the right extraction sequence and protecting fragile harvests (cards that reduce yield if misplayed). Rewards scale by quality tiers (basic extract -> rare tincture) rather than simple HP removal.

- Provisioning (gathering/crafting subtype): focuses on combining ingredients and time/temperature/chemical mechanics represented with token counters. Ingredient cards have freshness and flavour tokens; Provisioning action cards consume ingredient tokens and may consume or produce Ration tokens depending on specific card rules. Sequencing and timing cards (HeatUp, Stir, Rest, Distill) modify quality multipliers. Provisioning produces Ration tokens, reagents, consumables, or buffs rather than raw loot.

- Learning / Research (interaction subtype): focused on research progression rather than a one-off encounter. Players maintain a Research deck of potential projects; when choosing a new project the player picks the next research card from a random set of X cards (for example, 3) — the chosen research becomes the active project and must be completed before another can be selected. The number of candidate research cards presented when choosing a new project is determined by the "max research choice" token; players start a new game with 2 such tokens.

- The Learning encounter is the act of advancing the active research: play Study, Experiment, and Tool cards and spend tokens such as Insight and Thesis to make progress. Research projects may require specific sequences, repeated successful plays, or particular tool/knowledge cards to reach completion.

- Example - Research tradeoff: during a Learning encounter the player may opt to convert immediate progress into a temporary reduction of their "max research choice" token (for example, reduce it by 1). This grants instant advancement on the active project but reduces the number of candidate research cards presented the next time a project must be chosen; the reduction persists until the player spends resources or actions to increase the token again, creating interesting trade-offs between short-term gains and long-term flexibility.

- New potential research cards are obtained as rewards out in the world (from area/treasure drops, merchants, or special encounters) or as unlocks tied to completing other research projects; this creates a flow of new research options that can expand player capabilities over time.

- Completing research unlocks rewards tied to the project (new recipe cards that are added to the Library, improved yields for certain gathering decks, or permanent bonuses). Degree of progress and quality of plays determine outcome tiers, so better-designed Learning encounters yield stronger unlocks.

Research cards, modifier deck, and variant creation

- Research cards: a card in the Research deck carries the temporary type "research" until it is completed. When a research card is finished it is converted into its final type (for example a combat card) and a copy of the finished card joins the Library under that type; the finished card is thus available for decks and library pulls going forward.

- Replenishing research: after a research card is completed, a new research card is created to replace it. The new research card will have the same base discipline/type as the one just completed (if you researched a combat card, the new research candidate will also be combat-flavored).

- Modifier Deck workflow: new research variants are produced by combining a base research card with modifiers drawn from a Modifier Deck. Replacement research cards are generated immediately and synchronously as part of research completion: draw X modifier cards from the Modifier Deck (X is determined by the Variant-Choice token) and choose up to Y of them to attach (Y is determined by the Affix-Picks token). All draws, choices, and any random rolls are deterministically derived from the game's single initial seed and recorded so the generation is deterministic and reproducible.

- Combining: the chosen modifiers (example affixes: Heavy, Showy, Corruptive-Edge) determine which mechanical elements get added to the new research card; random rolls (derived from the game's single initial seed) decide numeric values for those elements and are biased by Refinement and Stability tokens which improve result quality or reduce variance. The final Variant Card is then recorded (game seed and choices logged) and added to the Library; rules separately govern replenishing the Research deck with a new research-card candidate of the same discipline if desired.

- Tokens and balancing: Variant-Choice (controls X), Affix-Picks (controls Y), Refinement (improves roll outcomes), and Stability (reduces variance) are the primary tokens used in this flow. This keeps research generative but constrained: designers can tune rarities in the Modifier Deck and the costs for increasing X/Y/Refinement to balance exploration vs. exploitation.

- Example: finish researching a Basic Axe (combat research card) → draw 3 modifiers (X=3) from the Modifier Deck → choose 1 (Y=1) such as Showy (+Renown-on-hit, small combat penalty) → roll damage/cleave values biased by Refinement → the resulting Variant Axe is added to the Library and the Research deck is replenished with a new combat research card created by repeating the Modifier Deck process.

- Reproducibility: all draws and random rolls in this creation process are deterministically derived from the game's single initial seed and recorded so the exact Variant outcomes can be reproduced if needed.

- Scouting (system / reconnaissance): Scouting is not an independent encounter type; it is the post-resolution step of every encounter lifecycle. After an encounter resolves (win or loss), scouting-related effects may be applied to update the Area deck and future draws: scouting cards, actions, or tokens influence how replacement encounters are generated and how future encounter choices are presented.

  Mechanics and flow:
  - Replacement pipeline: immediately after any encounter resolves it is removed from its Area deck and replaced by a new encounter of the same base type with affix modifiers; this replacement is part of the encounter lifecycle and occurs regardless of outcome.
  - Affix generation: to build the replacement encounter, draw X affix-candidate cards from the Modifier Deck and choose up to Y of them to attach as affixes. The scouting-candidate-pool size X and scouting-pick-count Y are influenced by scouting-related tokens (they use the Variant-Choice / Affix-Picks semantics as research flows, and scouting bonuses may temporarily increase these values). The Modifier Deck here is the same shared Modifier Deck used by research replacement workflows.
  - Replacement choices and preview: when deciding which encounter to attempt next, a scouting-related token (for example Foresight) controls how many encounter options a player may draw/preview from the Area deck; scouting can therefore both influence replacement creation and encounter selection scope.
  - Risk and cost management: scouting effects are modulated by spends (Rations, stealth-like tokens, or scouting-specific tokens) to bias draws toward higher-risk/higher-reward variants or to enable parameter upgrades. All draws and choices in this pipeline are deterministic and recorded (derived from the game's single initial seed).

  This system reuses the research/replacement workflow and makes area evolution an explicit part of every encounter's lifecycle.

- Mining (gathering subtype): focuses on discipline wear and extraction. Uses a single-deck resolution (ore_damage vs durability_prevent tradeoff) with no phases. Future refined versions (Step 8.5) will add tiered rewards, tier-increasing card effects, and insight token generation.
- Woodcutting (gathering subtype): focuses on rhythm and pattern-building for greater yields. There is no enemy deck. Player plays up to 8 Woodcutting cards (starting hand of 5, drawing 1 per play). Each card has a ChopType (LightChop, HeavyChop, MediumChop, PrecisionChop, SplitChop) and a numeric value (1-10). Cards can have multiple types and values but start with 1 of each. After all plays (or the player chooses to stop early), the best matching pattern (poker-inspired: flushes, straights, pairs, etc.) determines Lumber reward. Pattern rarity has significant multiplier impact to reward playing more cards. Each card costs a small fixed durability; WoodcuttingDurability depletion is a loss condition. The strategic tension is between building patterns and conserving durability.
- Fishing (gathering subtype): focuses on numeric precision. Each encounter defines a valid_range, max_turns, and win_turns_needed. Each round the player and enemy (fish) play numeric cards; the difference (clamped ≥ 0) must fall within the valid_range to count as a "won" turn. Win enough turns before max_turns to win. Every card costs durability; FishingDurability depletion is a second loss condition.

Design consequences and examples:
- Different card pools: each subtype has bespoke card families (Knowledge cards, Tool cards, Time cards, Recon cards) so card synergies are meaningful and specific to the activity.
- Different token uses: tokens serve different roles per subtype (Stamina as cross-discipline cost currency, Insight in learning, MiningDurability/WoodcuttingDurability/HerbalismDurability/FishingDurability as discipline-specific HP pools), keeping economies distinct while interoperable when appropriate.
- Different victory metrics: combat uses HP/objective removal; crafting/gathering use quality, yield, or connection strength; learning and the scouting system prioritize gathered information, unlocked library cards, and improved future encounter options.
- All interactions remain deterministic/replayable via seeds: shuffles and deterministic resolutions preserve reproducibility while enabling diverse mechanical flavors.

### Encounter win/loss patterns

Five distinct win/loss patterns have emerged across implemented encounter types. Future encounter types should reuse or explicitly extend these patterns:

1. **HP depletion** (Combat, Mining): reduce the encounter target's HP to 0 to win. Lose if the player's relevant HP/durability pool reaches 0. Binary outcome.
2. **Card narrowing** (Herbalism): win when exactly 1 enemy card remains; lose when 0 remain (over-harvested). The player must be precise — too aggressive play causes a loss. Second loss condition: HerbalismDurability ≤ 0.
3. **Degree of success / pattern evaluation** (Woodcutting): the encounter always "completes" but the quality of the win (Lumber reward) depends on the best pattern formed from the played cards. Lose only if WoodcuttingDurability ≤ 0 during play.
4. **Card-subtraction with valid-range targeting** (Fishing): win enough rounds (result within valid_range) before max_turns exhausted. Lose if turns run out or FishingDurability ≤ 0.
5. **Threshold / quality** (future crafting/provisioning): success if final quality meets recipe threshold; fail otherwise. Not yet implemented.

### Enemy behavior patterns

Encounter types use distinct enemy behavior patterns that future types may reuse:

- **Draw and play** (Combat, Mining): enemy has a deck, draws cards, and plays from hand each turn. Discard recycling when deck is empty.
- **Fixed hand, no draw** (Herbalism): enemy starts with a fixed hand and never draws. The hand shrinks as the player removes cards via characteristic matching.
- **Fixed hand, consume on play** (Fishing): enemy has a fixed hand of cards, plays one randomly each turn. The played card is consumed (moved to discard), so the hand shrinks over time.
- **No enemy deck** (Woodcutting): no enemy or node deck at all. The encounter is purely about the player's card choices. Challenge comes from hand management and pattern construction.

### Stamina as cross-discipline cost currency

Stamina is intended as the shared cost currency across all disciplines. It is earned through rest encounters and specific card effects, and spent as an optional cost for powerful card plays. Unlike discipline-specific durability tokens (which are encounter-operational pools), Stamina is a strategic resource the player manages across encounters. Future steps (9.2, 9.3) will implement Stamina costs on cards and rest encounters for recovery.

## Special tokens and cross-discipline currencies

This subsection is the authoritative token reference. Each token type below lists how it is typically earned, where it may be spent, and whether it can carry structured payload data.







- Variant-Choice: controls X (how many modifier cards are drawn when generating research replacements); earned rarely via milestones or special rewards; numeric.

- Affix-Picks: controls Y (how many modifiers may be chosen); very expensive to increase and earned via deep progression; numeric.

- Refinement: crafting token used to bias or re-roll numeric affix values; numeric and spent during Craft encounters.

- Stability: crafting token that reduces variance on affix rolls; numeric and spent during Craft encounters.

- Momentum: combat token gained by chaining offensive plays or foregoing defence; spent to trigger follow-up or combo effects.



- Corruption (and Purity): Corruption earned by forbidden actions; Purity earned by restraint and quests. Purity may be spent to purge Corruption. Corruption influences world reactions and may gate content.



- Key tokens / Special-purpose tokens: awarded by milestone chains; tokens may carry metadata (key_id) and are spent to unlock elite challenges. Rare and unique within a single-player save.

- Cost-reduction cards/effects: not tokens, but discipline cards or Library effects that reduce resource or token costs for specific encounters (e.g., a consumable that reduces a Craft's resource requirement).

General rules

- Tokens may be numeric counters or structured JSON objects (for example CurrentResearch or Key tokens) and are persisted in save state.
- Each token type must have clear earn rules, caps/decay/refresh semantics, and a single authoritative set of usages; all state mutations go through the POST /action endpoint to preserve reproducibility.
- Tokens are primarily single-player save-scoped; they are not account-shared or multiplayer objects.

## Trading and merchants

- Merchants are modelled as a MerchantOffers deck which yields shop offers. Every merchant interaction deterministically draws `MerchantOfferPool` offers from the MerchantOffers deck and presents them to the player; the player selects one offer to pursue. Next the system draws `MerchantBargainDraw` cards from the Barter deck and presents them as potential modifiers; the player may choose up to `MerchantLeverage` of those Barter cards to modify the selected offer. Each Barter card can change `offered_token`, `requested_token`, `rate`, `fee`, or attach conditions (for example time-limited availability or additional item requirements). Barter cards may also modify `MerchantOfferPool`, `MerchantBargainDraw`, or `MerchantLeverage` either for the current interaction or persistently; all draws, token changes, player choices, and RNG seeds are recorded in the actions log so encounters remain reproducible.

## Example turn flow (compact)
- Start turn: shuffle/draw driven deterministically from the game's single initial seed, draw to hand from player decks.
- Play actions: use Resource cards to activate Attack/Mining/Crafting cards.
- Resolve encounter: enemy or resource card acts (enemy attacks, ore resists, merchant updates offers).
- End turn: discard, apply persistent effects, record state.

## Reproducibility and single-game seed
- The game is initialized with a single seed at new-game start; every deck shuffle and random selection is produced deterministically from that initial seeded RNG; recording that seed and deterministic decision inputs allows exact replay of any session, combat, or crafting attempt.

## Early game and progression

The game starts intentionally simple: players begin with basic Attack, Defence, and Resource decks, a single Area deck containing a few simple enemies (each with small, focused decks), and a Research deck populated with straightforward projects (examples: Axe, Shield). Early research options teach core mechanics and unlock new card types and deck possibilities as the game progresses; research projects themselves become new gameplay loops that can be expanded.

The Area deck is the primary loop and lure: visiting areas is the only reliable way to encounter new opportunities (combat, gathering, learning/research, crafting, merchants). New research candidates, resources, and library cards are primarily gained from area encounters or as unlocks from completed research, so players are always incentivized to return to area encounters to advance and acquire rewards.

This design lets the game grow from simple beginnings to rich systems while preserving the "everything is cards and tokens" model and keeping the Area deck as the central pathway for progression.

## Interface and API

- The game is intentionally pure text-based and exposed entirely via a REST JSON API so it remains playable by humans who can issue and read JSON requests/responses.
- All mutable interactions are performed through a single actions endpoint (e.g., POST /action) where each operation is a concise JSON object describing the intended action; this endpoint is the canonical way to interact with and mutate game state. The current player actions are: NewGame, EncounterPickEncounter, EncounterPlayCard, EncounterApplyScouting, EncounterAbort.
- An append-only actions log endpoint (for example GET /actions/log) exposes all player actions in chronological order. When paired with the game's initial seed used for RNG, the full chronological action list is sufficient to deterministically reproduce the exact game state at any point. Only player actions are recorded; internal operations (token grants, card movements) are deterministic consequences of the seed and player actions.
- Other endpoints (for example /library/cards, /encounter, /encounter/results, /player/tokens, /actions/log) provide read-only JSON access to game data; the actions endpoint is the only endpoint that performs state changes. The /library/cards endpoint supports `?location=` and `?card_kind=` query filters and returns cards with their IDs.
- The project's OpenAPI specification will include thorough tutorials, documentation, and guided examples (end-to-end flows, example requests/responses, recipe/crafting walkthroughs) so integrators, designers, and QA can learn and exercise the API directly from the spec.

## Card location model and counts

- Canonical Library entry: each Library entry is a single authoritative object combining the immutable card definition (id, type, base properties, affix definitions, and metadata) with the player's exclusive copy counts. The Library entry therefore represents both the card's definition and how many copies the player currently owns in each location.

  - definition: immutable card identity and properties (card_id, type, base stats, list of affix descriptors, rarity, tags).
  - counts: an exclusive counts tuple describing where player copies reside.

- Single deck per type: each card type maps to exactly one deck-type (Attack cards belong to the Attack deck, Defence cards to the Defence deck, etc.). For each deck type there is only one player-owned deck instance. Deck-type constraints determine which Library cards may be added to which deck.

- Exclusive counts and compact representation: the Library entry's counts tuple is ordered as [library_count, deck_count, hand_count, discard_count] (this list may be expanded later to include deleted, equipped, or other locations). Counts are exclusive — a single physical card instance is represented in exactly one location and contributes to exactly one count.

- New-definition uniqueness: when a crafted or researched outcome changes the card definition (for example different affix numeric values produce a distinct card definition), a new Library entry must be created for that distinct definition with its own counts. Library entry identity is therefore based on the card definition rather than a human-visible name.

- Research cards in the Library: Research-card candidates are present in the Library catalog and have counts like any other entry; while a research card carries the temporary type "research" it remains a Library entry and thus contributes to counts. When the research is completed the Library entry is updated to the final type and its definition is frozen.

- Example: a count vector of [4,2,1,1] on the Library entry for "Basic Axe (v2)" means the player has 8 copies total: 4 are stored in the Library inventory, 2 are currently in the Attack deck, 1 is in the player's hand, and 1 is in the discard pile.

- Lifecycle movements: common operations move a card between these locations (Library → Deck when added to a deck; Deck → Hand when drawn; Hand → Discard or Deleted when played or consumed). All such movements are deterministic and recorded in the actions log so state can be reconstructed given the game's initial seed and the action list.

## Goals and milestones

- Trackable goals: every mutating operation increments counters so players can aim to hit milestones within a fixed number of operations. A Milestones deck contains hardcoded challenge cards (structured similarly to Area encounters) that pose focused challenges and require spending discipline-specific Insight to attempt.

- Unique modifier and card acquisition: completing milestone challenges is the primary (and exclusive) source of new Modifier cards for the Modifier Deck; challenges can also reward unique cards that cannot be crafted or researched elsewhere (once obtained they become Library items that can be targeted by later crafting or research flows).

- Challenge life cycle and escalation: when a challenge is defeated it is replaced by a new, tougher challenge with greater reward. Toughness should evolve not merely by numeric scaling but by introducing new mechanics, constraints, and required playstyles — for example forcing limited hand size, changing turn order, applying persistent mutators, or requiring cross-discipline synergies.

- Ideas for meaningful continuous progression (non-exhaustive):
  - Rule mutators: new challenges introduce unique rule changes (e.g., reversed initiative, delayed resource refresh, limited card types allowed) that require players to adapt their decks and tactics.
  - Cross-discipline requirements: some milestones require using two or more discipline decks in sequence (e.g., scout an area then perform a specialized craft under time pressure) to reward broader investment.
  - Branching challenge trees: completing certain challenges unlocks branches of related challenges that grant access to new affix families or modifier categories.
  - Rotating modifiers and seasonal variants: periodically rotate challenge modifiers so the same challenge can demand different approaches over time.
  - Reward choice and meaningful trade-offs: offer multiple reward options (choose one) such as a rare affix, a unique card, or an increase in Variant-Choice/Affix-Picks caps, forcing players to decide between immediate power or long-term progression.
  - Meta and chain challenges: chains that require a series of wins to unlock an elite modifier or an otherwise unobtainable affix; failing mid-chain increases future costs or changes the chain path.
  - Persistent world effects: some challenges unlock global changes (new area decks, merchant inventory updates) that alter future encounter composition.
  - Milestones designed for single-player progression: operation-limited challenges track personal performance for local goals or seasonal single-player modes; multiplayer or leaderboard features are out of scope.

- Design notes: initially the Milestones deck can be hardcoded to seed the system; over time new challenges can be procedurally generated or authored and inserted into the deck. Make sure Insight cost gating ensures players invest in discipline progression before attempting higher-tier milestones. Record all challenge attempts and results in the actions log so progression and competition are auditable and reproducible.

## Encounter templates by discipline

This section provides a canonical encounter template and concrete examples for each major discipline so designers and implementers share a consistent expectation of encounter data, phases, decks, tokens, rewards, and failure modes.

Encounter template (fields every encounter card provides)

- id: unique identifier
- type: discipline (Mining, Woodcutting, Herbalism, Combat, Fabrication, Provisioning, Research, Merchant) — Scouting is a cross-cutting reconnaissance/system (post-resolution) and not a standalone encounter discipline.


- stats / parameters: discipline-specific numeric values (HP/resilience, hardness, freshness, complexity, time-to-complete)
- modifiers: list of applied affixes or environmental modifiers
- entry_cost: tokens or resources required to attempt (optional)
- rewards: reward pool specification (materials, tokens, Variant-Choice chances, recipes)
- failure_consequences: canonical list of consequences and tokens applied on failure


Encounter lifecycle (includes post-resolution scouting/area-update step)

- Pre-start: all encounter fields are visible before committing to the encounter.
- Start: decks are bound to the encounter (encounter deck, reward deck, modifier pulls), and any entry_cost is consumed/locked; random draws and rolls derive deterministically from the game's single initial seed.
- Phases: encounters are resolved in named phases; a common minimal set: Setup → Player Phase(s) → Encounter Phase(s) → Resolution → Post-resolution area-update/scouting. Discipline-specific phases add nuance (for example, a "Preparation" phase for Provisioning or "Extraction" rounds for Mining).
- Actions: players take structured actions by playing discipline-specific action cards from their hand/decks, spending tokens, or triggering reactions. Each action maps to deterministic outcomes recorded in the actions log.
- Resolution: encounter finishes when win or loss conditions are met; rewards and post-encounter transitions (move to Discard, Deleted, Library additions) are applied and recorded. Immediately after resolution a post-resolution area-update/scouting step occurs as part of the encounter lifecycle: replacement encounters are generated (affix candidate draws and pick attachments), Foresight effects are applied, and any preview/selection options for the next encounter are computed and recorded.

Win / Loss semantics

- Each encounter declares explicit win (for example reduce encounter HP to 0, reach success threshold, complete X progress steps) and loss conditions (for example MiningDurability ≤ 0, Exhaustion ≥ cap, timeouts). Both are deterministic given the game's single initial seed and action log.
- Tie-breaking: encounters should define deterministic tie-break rules (fail ties, win ties, or last-action priority) to ensure reproducibility.

Difficulty progression between encounters

- Encounters of the same type scale difficulty using area/zone progression and probability distributions tuned by Variant-Choice, affix rarities, and designer-set difficulty curves. Difficulty may increase via: higher base stats, more modifiers, and harsher failure penalties.
- Procedural patterns: area decks can be composed with weighted tiers so later draws increase expected difficulty while still allowing occasional low-difficulty encounters for variance and pacing.

How the encounter differs from others

- Each discipline differentiates via: the decks used (action decks, resource decks), token economies involved, meaningful choices (timing vs sequencing vs sequencing+precision), and victory metrics (HP removal vs quality threshold vs extraction yield). The template below highlights per-discipline differences.

Concrete examples

1) Mining (gathering)

- Current simplified implementation (Step 8.1):
  - Encounter card fields: OreHealth (via ore_tokens), ore deck (DeckCounts), rewards (HashMap<Token, i64>).
  - Pre-start: all encounter fields are visible before committing.
  - Start: OreHealth token set on MiningEncounterState.ore_tokens; MiningDurability checked (persists from game start at 100).
  - Phases: none — single action per turn, no phase rotation.
  - Player actions: Play Mining cards (ore_damage + durability_prevent tradeoff). Player draws 1 mining card per play.
  - Decks: Player Mining deck, Ore deck (encounter-internal, uses DeckCounts).
  - Tokens: MiningDurability (persistent, game-start init), OreHealth (encounter-scoped), Ore (reward).
  - Win: OreHealth ≤ 0 → grant reward tokens (Ore: 10). Loss: MiningDurability ≤ 0 → PlayerLost, no penalties.
  - EncounterAbort available (marks as PlayerLost, no rewards/penalties).

- Future refined version (Step 8.5 — end-state vision):
  - Tiered rewards: Ore T1, T2, T3. Player card effects increase encounter tier (harder gameplay, higher reward tier).
  - Tier 2: moderately hard. Tier 3: very hard. Difficulty increases via gameplay involvement, not just durability removal.
  - Insight tokens: T1, T2, T3 — generated by discipline-specific insight card effects.
  - Stamina replaces Rations as the cost currency for boosts.
  - Difficulty progression: successive nodes in the same area increase base hardness; variant modifiers increase with area level.
  - Distinctive features: Mining emphasizes repeated extraction rounds, discipline wear, and yield-quality trade-offs.
  - Rewards: Ore (tiered), Gems, potential recipe fragments.
  - Failure consequences: reduced MiningDurability, resource loss, Exhaustion gain.
  - Win/Lose: win by depleting node HP or completing required extraction; lose if MiningDurability depletes.

2) Woodcutting (gathering)

- Current simplified implementation (Step 8.3):
  - Unique mechanic: rhythm-based pattern matching for greater yields. NO enemy deck.
  - Player Woodcutting cards have: a ChopType (LightChop, HeavyChop, MediumChop, PrecisionChop, SplitChop), a numeric chop_value (1-10), and a durability_cost (fixed small cost ~1).
  - Cards can have multiple types and values, but initial cards have 1 of each.
  - Player starts with hand size 5 and plays up to 8 cards total. Drawing 1 new card per play.
  - After all plays (or the player chooses to stop early): evaluate the played cards for the best matching pattern (poker-inspired: flushes of same type, straights of sequential values, pairs/triples/quads, full houses, etc.) and reward Lumber tokens accordingly. Early stop is NOT an abort — the pattern of all cards played so far is still evaluated and rewards granted. Durability cost is only paid for cards actually played.
  - Only the best pattern is used. There are always some simple patterns that match, so the player always gets some reward. Pattern rarity has significant impact on multiplier to reward playing more cards.
  - Win: always wins after all cards are played or player stops early (pattern determines reward amount). Loss: WoodcuttingDurability ≤ 0 during play → PlayerLost.
  - WoodcuttingDurability initialized at game start (100), persists across encounters. EncounterAbort available.
  - Note: The starting library includes cards for LightChop, HeavyChop, MediumChop, and PrecisionChop. SplitChop has no starting cards — it is intended to be unlocked through crafting/research in later steps.

- Future refined version (Step 8.5 — end-state vision):
  - Tiered rewards: Lumber T1, T2, T3. Player card effects increase encounter tier (harder gameplay, higher reward tier).
  - Tier 2: moderately hard. Tier 3: very hard. Difficulty increases via gameplay involvement, not just durability removal.
  - Insight tokens: T1, T2, T3 — generated by discipline-specific insight card effects.
  - Stamina replaces Rations as the cost currency for boosts.
  - Difficulty progression: encounter modifiers that restrict playable types or increase durability costs.
  - Rewards: Lumber (tiered), occasional special wood components.
  - Failure: WoodcuttingDurability loss, reduced yield.
  - Win/Lose: pattern-based reward scaling; lose if WoodcuttingDurability drops to 0.

3) Herbalism (gathering)

- Current simplified implementation (Step 8.3):
  - Mechanic: card-characteristic matching (unique among gathering disciplines).
  - Enemy (plant) starts with X cards on hand. The plant does NOT draw more cards.
  - Each enemy card has 1-3 characteristics from a small enum (e.g., Fragile, Thorny, Aromatic, Bitter, Luminous).
  - Player plays Herbalism cards that target characteristics. Playing a card removes all enemy cards that share at least one characteristic with the player's card.
  - Win: exactly 1 enemy card remains → that card becomes the reward, plus Plant tokens granted. Loss: 0 enemy cards remain (over-harvested — too aggressive), OR HerbalismDurability ≤ 0 (durability depleted — each player card has a durability_cost applied immediately on play).
  - HerbalismDurability initialized at game start (100), persists across encounters. Player draws 1 card per play.
  - EncounterAbort available (marks as PlayerLost, no rewards/penalties).
  - Tokens: HerbalismDurability (persistent), Plant (reward material token).

- Future refined version (Step 8.5 — end-state vision):
  - Tiered rewards: Plant T1, T2, T3. Player card effects increase encounter tier.
  - Tier 2: moderately hard. Tier 3: very hard. Difficulty increases via gameplay involvement, not just durability removal.
  - Insight tokens: T1, T2, T3 — generated by discipline-specific insight card effects.
  - Stamina replaces Rations as the cost currency for boosts.
  - Difficulty progression: rarer plants require longer sequences and have higher contamination risk.
  - Distinctive features: precision and sequence matching; failures often reduce potency rather than causing total loss.
  - Rewards: Plant (tiered), Reagent, Tinctures, recipe unlocks.
  - Failure: lower potency yield, chance of harmful contamination token, small Exhaustion.
  - Win/Lose: win by completing sequence with sufficient potency; lose if contamination exceeds threshold or sequence breaks irreparably.

4) Fishing / Foraging (gathering)

- Current simplified implementation (Step 8.4):
  - Numeric card-subtraction mechanic: Each fishing encounter defines a valid_range (min, max), max_turns, and win_turns_needed.
  - Each round the player plays a numeric card first, then the enemy (fish) plays a numeric card. The two values are subtracted: result = (player_value - fish_value).max(0). If the result falls within the valid_range (inclusive), the turn counts as "won".
  - Win: player wins win_turns_needed rounds before max_turns are exhausted → grant Fish tokens. Loss: max_turns exhausted without enough wins → PlayerLost. OR FishingDurability ≤ 0 → PlayerLost.
  - FishingDurability initialized at game start (100). Every card played has a small durability_cost. Player draws 1 card per play.
  - EncounterAbort available.
  - Fish deck behavior: the enemy (fish) has a fixed hand of numeric cards, shuffled at encounter start using the seeded RNG. Each turn the fish plays one card randomly from its hand. The played card is consumed (moved to discard), so the fish hand shrinks over time. This is the "fixed hand, consume on play" enemy behavior pattern.
  - Tokens: FishingDurability (persistent), Fish (reward material token).

- Future refined version (Step 8.5 — end-state vision):
  - Tiered rewards: Fish T1, T2, T3. Player card effects increase encounter tier.
  - Tier 2: moderately hard. Tier 3: very hard. Difficulty increases via gameplay involvement, not just durability removal.
  - Insight tokens: T1, T2, T3 — generated by discipline-specific insight card effects.
  - Stamina replaces Rations as the cost currency for boosts.
  - Difficulty progression: rarer spawns and harder fish decks with higher numeric values.
  - Distinctive features: numeric precision and timing mechanics; rewards often consumables or provisioning ingredients.
  - Rewards: Fish (tiered), Foodstuffs, Ingredients, occasional rare items.
  - Failure: no yield and small Exhaustion.
  - Win/Lose: win by meeting win_turns_needed within max_turns; lose if turns exhausted or durability depleted.

5) Combat (enemy encounter)

- Encounter fields: HP, attack_pattern, defence_profile, scripted_triggers, rewards_on_defeat.
- Pre-start: visible enemy type, base stats (HP, Attack, Defence), and any known special attacks.
- Phases: Setup → Player Turn → Enemy Turn → Repeat → Resolution → Post-resolution area-update/scouting.
- Actions: Play Attack/Defence/Resource cards, spend Momentum, use Reaction cards.
- Decks: Player Attack/Defence/Resource decks, Enemy deck, Token pools.
- Tokens: Momentum, Exhaustion, Discipline Durability does not apply (combat has own resources), Rations as stamina.
- Difficulty progression: enemies in areas gain modifiers, patterns grow more complex.
- Distinctive features: adversarial play with reactive scripting and tempo importance.
- Rewards: Loot, Renown, potential recipe drops.
- Failure: player defeat (character HP depleted).
- Win/Lose: win by reducing enemy HP to 0; lose by player HP reaching 0. Players cannot abandon or retreat from combat once started (EncounterAbort returns 400 for combat encounters).

6) Fabrication / Weaponcraft (refining / crafting encounter)

- Encounter fields: recipe_requirements (materials and token costs), required_sequence_steps, quality_thresholds.
- Pre-start: visible recipe name, base cost, visible optional modifiers that can be attempted.
- Phases: Setup → Work Rounds (apply hammering, tempering, quenching actions) → Quality Rolls → Resolution → Post-resolution area-update/scouting.
- Actions: Play Discipline cards (Hammer, Temper, Anneal), spend Refinement/Stability to bias rolls or reroll affix values, spend refined materials and Thesis/Insight if used for special options.
- Decks: Fabrication deck, Inventory/Material decks, Library recipe reference, Reward deck for outcomes.
- Tokens: Refinement, Stability, Rations optional as stamina, Discipline Durability (heavy work can deplete fabrication durability pool).
- Difficulty progression: higher-tier recipes require more refined inputs, higher Refinement/Stability investment, and longer action sequences.
- Distinctive features: deterministic quality roll pipeline with seeded RNG; crafting produces new Library card definitions and is the main economy sink.
- Rewards: new Library card copies, possible bonus affixes, and recipe improvements.
- Failure: resource loss, Exhaustion, possible tool/discipline Durability reduction, lower quality or outright failed craft.
- Win/Lose: success if final quality meets recipe threshold; fail otherwise.

7) Provisioning (refining / crafting)

- Encounter fields: reagent_requirements, ingredient_list, volatility/freshness, stabilization_difficulty, temperature_profile.
- Pre-start: all encounter fields are visible.
- Phases: Setup → Mix/Prep/Heat/Distill/Cook Rounds → Stabilization/Plating → Resolution → Post-resolution area-update/scouting.
- Actions: Play Mix, Heat, Distill, Stir, Rest, Season cards; spend Refinement/Stability and Thesis for special experiments; timing and sequencing are important.
- Decks: Provisioning deck, Inventory/Ingredient decks, Library.
- Tokens: Reagent/Tincture, Ingredients, Rations, Refinement/Stability, Insight/Thesis for research synergies.
- Difficulty progression: rarer or exotic recipes require tighter stabilization windows and more precise timing.
- Distinctive features: volatile processes and time/temperature sequencing where sequencing and timing heavily matter; produces consumables and reagents rather than durable Library entries (though recipes can be unlocked).
- Rewards: reagents, consumable cards, new recipes.
- Failure: ruined reagents, volatile failures (Exhaustion, Corruption risk), reduced quality.
- Win/Lose: success if final product meets quality thresholds.
- Rewards: Rations, consumables, buffs, recipe unlocks.
- Failure: spoiled dish (lower quality Rations) or minor Exhaustion.
- Win/Lose: success if final dish meets quality thresholds.

10) Research / Learning (long-form encounters)

- Encounter fields: project_id, required_progress, candidate_variants, research_prereqs.
- Pre-start: visible project summary and base requirements; initial choice set presented when starting a new project.
- Phases: Setup → Study Rounds (play Study/Experiment cards) → Milestone Checks → Completion.
- Actions: Play Study, Experiment, Tool cards; spend Insight, Thesis, Variant-Choice/Affix-Picks as configured.
- Decks: Research deck, Library, Modifier Deck for variant draws.
- Tokens: Insight, Thesis, Variant-Choice, Affix-Picks, Refinement/Stability for biasing rolls.
- Difficulty progression: later research projects require more progress and rarer resource spends.
- Distinctive features: long-form progression with persistent CurrentResearch state and research replacement workflows on completion.
- Rewards: new recipes, variant cards added to Library, Modifier Deck entries.
- Failure: project abandonment penalties, resource loss, or reduced outcomes.
- Win/Lose: success on reaching required_progress; failure if player abandons or fails milestone constraints.

11) Scouting / Recon (system)

- Lifecycle placement: scouting is applied as the post-resolution area-update step of the encounter lifecycle (Setup → Player Phase(s) → Encounter Phase(s) → Resolution → Post-resolution scouting/update).
- Decks involved: Area deck and Modifier Deck (scouting reuses these decks; there is no separate persistent 'Scouting' deck).
- Tokens: Foresight, Scouting candidate pool size, Scouting canditate pick size, Rations, Stealth-like tokens.

12) Merchant / Trading interactions

- Encounter fields: offer_pool, barter_options, fees.
- Pre-start: visible offers and any required bargaining tokens.
- Phases: Setup → Offer Selection → Bargain Rounds → Resolution → Post-resolution area-update/scouting.
- Actions: Choose offer, play Barter cards, spend Renown/Renown-like tokens to unlock options.
- Decks: MerchantOffers, Barter deck, Player Barter/Resource decks.
- Tokens: Renown, Trade tokens, Loot.
- Difficulty progression: merchant inventory rotates and rarer offers appear as Renown or Milestone requirements are met.
- Distinctive features: decision-heavy with optional bargaining minigame.
- Rewards: cards, materials, special deals.
- Failure: missed deal, lost bargaining tokens, or fees.
- Win/Lose: success by securing a desired offer; failure if bargaining collapses.

Cross-discipline notes

- Consistency: the game's single initial seed and all deterministic decisions must be recorded in the actions log so sessions can be exactly replayed.
- Shared mechanics: phases and action mapping remain consistent across disciplines to make client implementations straightforward.
- Difficulty tuning: use area-tiering and modifier rarities to tune pacing rather than ad-hoc encounter-level scaling.

## Open-ended sandbox

- The game is intentionally designed as an open-ended sandbox rather than a single finite campaign: there will be no final, absolute win or loss state. Players pursue personal goals, collections, and milestones; milestones function as optional, player-paced objectives and progression markers rather than an end condition. Milestones provide pacing, challenge, and meaningful rewards, but players are free to continue exploring, crafting, and experimenting indefinitely.

## Endpoint organization

Public endpoints (without /tests prefix) represent the stable gameplay API. Endpoints under /tests/* are temporary testing utilities and may be removed as implementation progresses. 

The only public endpoint that can mutate any data is the /action endpoint, every other public endpoint is a GET endpoint. 

## Automatic test setup 

Favor tests that spin up the server and verifies a test case by calling the public endpoints. It is okay that the test gets a bit long in an effort to get to a specific point: consider helper functions to wrap longer series of calls. Thisis fine becuase all changes are in memory so should go fast. It also ensures that all cases are reachable on the server and it makes it easy to review how the endpoints are used. 

## Entity ids 

If an id is a string then it has to be a UUID else favour using unsigned integers. 

## Closing

This vision emphasises a single consistent modelling approach: everything is expressed as decks and tokens, interactions are turn-based, and the entire system is reproducible via seeds. This makes the game predictable for designers, testable by QA, and open to emergent gameplay within a constrained, deterministic system.

