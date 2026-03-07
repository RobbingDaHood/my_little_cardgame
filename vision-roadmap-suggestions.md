# Vision & Roadmap Suggestions — Step 10 Research & Crafting Fixes

## What was implemented

- **Research encounters (Step 10)**: Full research lifecycle — discipline tags, Insight card effect, research encounter with choose/progress/complete actions, deterministic candidate generation, persistent research state.
- **Crafting fixes**: Card deduplication, merged conclude/auto_conclude into `finish_active_craft()`, abort blocking during active craft, variable cost distribution (2–4 tokens, Fisher-Yates, 75% cap).
- **Enemy card effect refactoring**: All enemy card types now carry `effects: Vec<ConcreteEffect>`. EnemyCardEffects moved to discipline modules. `validate_card_effects()` validates across all encounter types.
- **Discipline enum**: `Discipline` enum (8 variants) with `discipline_tags` on CardEffect entries and `card_effects_for_discipline()` filtering.

## Suggestions for vision.md

- The vision describes "Durability card effects are generalized — the discipline context determines which durability pool is affected" but this was deferred. Consider either updating the vision to reflect the per-discipline approach as the long-term design, or creating a specific roadmap item for durability generalization.
- Consider documenting that researched cards are currently always Attack type — the vision implies discipline-appropriate card kinds. Either update the vision to note this simplification or add a roadmap item.
- The vision mentions Insight as "per-discipline" but the current implementation uses a single shared Insight token. Clarify whether per-discipline Insight pools are still planned or if the shared pool is the intended design.
- Consider documenting the Discipline enum in the "Core gameplay elements" section since it's now a first-class type used across the codebase.

## Suggestions for roadmap.md

- Add a roadmap item for **Insight in gathering encounters** — currently only Combat and Rest process Insight effects. This limits the strategic depth of Insight cards in Mining, Herbalism, Woodcutting, Fishing, and Crafting.
- Add a roadmap item for **discipline-to-card-kind mapping in research** — researched cards should produce the appropriate card kind (Defence, Resource, Mining card, etc.) based on the discipline being researched.
- Consider a roadmap item for **research encounter accessibility** — the research encounter starts in the deck (not hand), making it hard to discover early. Options: start in hand, add a dedicated "research" action outside encounters, or increase draw probability.
- The crafting abort-blocking behavior (can't abort during active craft mini-game) should be mentioned in the vision's crafting section as a design decision.
- Consider adding a roadmap item for **enemy card effect balancing** — now that all enemy types have registered effects, a balancing pass could ensure the effects create meaningful encounters across all disciplines.

## What was deferred / not yet implemented

- **Generalized durability effects**: Per-discipline durability tokens kept separate (MiningDurability, HerbalismDurability, etc.) instead of a context-sensitive generalized effect.
- **Non-Attack researched cards**: All researched cards are Attack cards regardless of discipline.
- **Insight in gathering encounters**: Only Combat and Rest encounters resolve Insight effects.
- **Research encounter in starting hand**: Currently in deck only, reducing early-game accessibility.
- **Stamina/Health cost crafting cards**: Roadmap says these should be usable but only CraftingToken is used as cost.
- **Card modification/enhancement**: Only creation (copy) is implemented for crafting.

### Step 9.6 — Mark as implemented with notes

The playable acceptance criteria are met:
- ✅ Can resolve a crafting encounter
- ✅ Produces a Library card copy visible via GET /library/cards
- ✅ Demonstrates cost evaluation based on card effects
- ✅ Crafted cards placed in library (never directly in decks)
- ✅ Single crafting encounter type proves the flow

Consider adding a follow-up sub-step:
> **9.6.1) Crafting card variety and balancing** — Add Stamina/Health cost crafting cards, tune material costs and enemy deck strength, add multiple crafting encounter tiers.

### Step 9.6 — Clarifications discovered during implementation

1. **"Replace a card between deck/discard pile and library"** — Implementation interprets this as a bidirectional swap: one card moves FROM deck/discard TO library, and another card moves FROM library TO deck. The roadmap already states this clearly.

2. **Crafting cost formula**: `base_cost = total_power * (1 + num_effects) / 4`, distributed equally across Ore/Plant/Lumber/Fish. This feels reasonable but could benefit from explicit design documentation in the roadmap about cost curves and scaling expectations.

3. **Token cost for starting a craft**: Currently `min(total_material_cost/100 + 1, remaining_tokens)` with a floor of 2 tokens. Higher-quality cards consume more crafting tokens to start, creating a meaningful choice between multiple cheap crafts vs. one expensive craft.

## Suggested vision.md updates

- The crafting discipline validates the "everything is a deck" design — enemy crafting cards use the same DeckCounts/HasDeckCounts infrastructure as combat, mining, etc.
- Consider mentioning crafting as the "economy sink" in the vision, since it's the primary way players spend gathered materials (Ore, Plant, Lumber, Fish) to improve their card collection.
- The three-action-within-one-encounter pattern (swap/craft/durability) is novel compared to other disciplines which have a single card-play loop. This pattern could be documented as a design template for future complex encounters.

---

# Vision & Roadmap Suggestions — Discipline Tags & Insight Card Effect

## What was implemented

- **Discipline enum**: New `Discipline` enum in `types.rs` with variants for all seven disciplines (Combat, Mining, Herbalism, Woodcutting, Fishing, Rest, Crafting).
- **discipline_tags on LibraryCard**: Every library card now carries `discipline_tags: Vec<Discipline>` indicating which research disciplines can use that card/effect. Encounter cards use `vec![]` (no discipline).
- **add_card updated**: `Library::add_card()` accepts discipline tags; all callers across 7 discipline files, game_state, endpoints, and tests updated.
- **card_effects_for_discipline()**: New method on Library filters PlayerCardEffect entries by discipline tag, enabling the future research encounter to query available effects.
- **CardEffectKind::Insight**: New variant `Insight { min, max }` that grants Insight tokens when played. Handled in `roll_concrete_effect`, combat `apply_card_effects`, and rest `resolve_rest_card_play`.
- **Shared Insight PlayerCardEffect**: Registered at library init with all discipline tags, enabling cross-discipline research discovery.
- **Insight Resource card in combat**: Players start with 2 Insight Resource cards in their combat deck (deck:2), offering a strategic trade-off between combat actions and Insight generation.

## Suggestions for vision.md

- Document the Discipline tagging system as the foundation for the research encounter: discipline tags determine which card effects a research encounter can discover/upgrade.
- The Insight token was already defined in TokenType; now it's actively produced by the Insight card effect. Vision should clarify Insight's dual sources: MilestoneInsight (from combat victories) and Insight (from Insight cards during encounters).
- The "Insight vs combat benefit" trade-off in combat Insight cards is a key design pattern — consider highlighting this in the vision as the model for future cross-system trade-offs.

## Suggestions for roadmap.md

- Mark Feature 3A (Discipline Tags) and 3C (Insight Card Effect) as implemented prerequisites for the Research encounter.
- Feature 3B (Generalized Durability) was explicitly deferred — keep it on the roadmap but note it's independent of the Research encounter.
- Gathering disciplines (Mining, Herbalism, Woodcutting, Fishing, Crafting) use inline effect systems that don't support ConcreteEffect-based Insight cards. A future task should add Insight token grants to gathering card effects (e.g., via TokenAmount gains) or migrate gathering to use ConcreteEffect.
- The card_effects_for_discipline method is ready but not yet consumed. The Research encounter implementation should use it to present discoverable effects.

## What was deferred / not yet implemented

- **Insight cards in gathering disciplines**: Mining, Herbalism, Woodcutting, Fishing, and Crafting cards use inline effect structs (MiningCardEffect, etc.) not ConcreteEffect. Insight cards were only added to Combat. Gathering disciplines could add Insight via TokenAmount gains in a future pass.
- **Feature 3B (Generalized Durability)**: Per-discipline durability tokens remain; a unified durability system was not implemented per the spec.
- **Research encounter itself**: The discipline tags and Insight effect are prerequisites; the actual Research encounter type, discovery mechanics, and upgrade flow are not yet implemented.

## What was implemented

- **Unified enemy effect references**: All enemy cards across all disciplines now carry `effects: Vec<ConcreteEffect>` referencing `EnemyCardEffect` entries in the Library, matching the pattern Combat already used.
- **New `effects` field** added to `OreCard`, `PlantCard`, `FishCard`, and `EnemyCraftingCard` as a parallel reference alongside existing fields (`damages`, `characteristics`, `value`, `increases`).
- **Per-discipline EnemyCardEffect registration**: Each discipline's `register_*_cards()` function now registers its own `EnemyCardEffect` templates before creating encounter cards. Combat's effects moved from `game_state.rs` into `combat.rs`.
- **Expanded validation**: `validate_card_effects()` now validates enemy card effects across Mining, Herbalism, Fishing, and Crafting encounters (previously only Combat).
- **15 new EnemyCardEffect entries**: 5 mining (light/durability/health damage), 2 herbalism (small/medium plant value), 4 fishing (low/medium/high/very-high fish value), 4 crafting (ore/plant/lumber/fish cost increase).

## Suggestions for vision.md

- Document the "every enemy card references an EnemyCardEffect" principle as a core architectural decision — it enables scouting to present effect previews for all encounter types, not just combat.
- The parallel field pattern (keeping both domain-specific fields and ConcreteEffect references) is a deliberate bridge design. Vision could note that over time, encounter resolution may migrate to using ConcreteEffect directly, at which point the legacy fields can be removed.

## Suggestions for roadmap.md

- Add a follow-up task for migrating encounter resolution logic to use ConcreteEffect-based effects instead of hardcoded fields (damages, value, increases, characteristics). This would complete the unification.
- The scouting system can now generate previews for all enemy card types. Consider adding a roadmap step for implementing scouting UI/API that leverages these effect references.
- Consider a future task to remove the legacy fields (`damages`, `value`, `increases`) once resolution logic is fully migrated to use the effect system.
- The EnemyCardEffect registration pattern per discipline keeps card data co-located with its resolution logic. If the number of effects grows significantly, consider a shared registry or data-driven configuration file.

---

# Vision & Roadmap Suggestions — Research Encounter & Crafting Fix Tests

## What was implemented

- **6 new scenario integration tests** covering the Research encounter system and crafting fixes:
  1. `scenario_research_encounter_full_loop` — Full flow: choose project (tier 1), select candidate, optional progress, conclude, scout.
  2. `scenario_research_choose_and_swap_project` — Choose project, select candidate, conclude, verify project persistence.
  3. `scenario_research_insufficient_insight` — Verify insufficient Insight error when choosing a project with 0 Insight.
  4. `scenario_research_abort` — Abort a research encounter, verify PlayerWon, verify scouting.
  5. `scenario_crafting_abort_blocked_during_active_craft` — Verify abort is blocked while a craft mini-game is active.
  6. `scenario_crafting_card_deduplication` — Verify crafting increments existing card's library count rather than creating a new card.
- **Helper functions** added: `research_encounter_ids`, `win_combat_and_scout`, `play_one_round_prefer_insight`, `deplete_encounters_until_research`, `start_game_accumulate_insight_and_pick_research`.
- All 40 tests pass (`make check` clean).

## Findings during implementation

### Critical: Research encounter card inaccessible in early game

The Research encounter card starts with `deck: 1, hand: 0`, while all other encounter types start with `hand: 2-4`. The encounter draw system (`encounter_draw_to_hand`) only draws from deck when the hand count is below Foresight (default 3). Since the starting encounter hand has ~21 cards across 7 encounter types, the research card can never be drawn until ~19 other encounters are consumed. This means:

1. **Players must play/abort 19+ encounters** before Research becomes available.
2. **Encounter cards don't recycle from discard to deck**, so the encounter hand depletes permanently.
3. This makes Research a very late-game encounter, which may not be the intended design.

### Critical: Insufficient Insight generation for Research

Research tier 1 costs 10 Insight (choose) + 20 Insight (progress) = 30 total. However, Insight generation is extremely limited:

1. **Combat Insight Resource cards** (2 copies, deck only) generate 1-5 Insight per play, but are rarely drawn because the Resource hand is always full (5/5 main Resource cards).
2. **Best case across 3 combats**: ~10 Insight (with seed 7777 and targeted Insight card play).
3. **No other encounter type generates Insight** — gathering disciplines don't have Insight card effects.
4. **MilestoneInsight (100 per combat win) is a different token** and cannot be used for research.

This means completing a full research project (30 Insight) is currently impossible in a single game.

## Suggestions for vision.md

- Clarify the intended Insight economy: how much Insight should be available per game, and what fraction should go to Research vs other potential uses.
- Document the "encounter hand depletion → new encounter types appear" mechanic as an intentional late-game progression system, or flag it as a gap to address.

## Suggestions for roadmap.md

- **High priority**: Move the Research encounter card to `hand: 1` (or `hand: 2`) so it's accessible from game start, consistent with other encounter types.
- **High priority**: Increase Insight generation — either increase Insight Resource card starting hand count, add Insight generation to gathering encounters, or make MilestoneInsight convertible to Insight.
- Consider adding a test endpoint `POST /tests/tokens` that sets token balances directly — this would enable comprehensive research flow testing without depending on the full combat→Insight pipeline.
- Add a follow-up task to write research completion tests once the Insight economy is rebalanced.
- The `play_one_round_prefer_insight` test helper revealed that Insight Resource cards (with 1 effect) are identifiable by effect count vs main Resource cards (2 effects). This heuristic may break if card designs change — consider adding explicit card metadata for testability.
