# Vision & Roadmap Suggestions

This file captures suggestions discovered during the post-Step-10 implementation batch. Items marked ✅ were addressed in vision.md and roadmap.md during this batch.

---

## Implemented in this batch (for reference)

The following items were implemented and documented in vision.md/roadmap.md:

- ✅ Per-discipline insight tokens (CombatInsight, MiningInsight, etc.) replacing shared Insight and MilestoneInsight
- ✅ Insight card effect processing in all 7 disciplines
- ✅ Generalized durability card effects (TokenType::Durability resolved at encounter time)
- ✅ ConcreteEffect migration for Mining (OreCard.damages removed) and Crafting (EnemyCraftingCard.increases removed)
- ✅ Cap behavior: caps limit gain, not total balance
- ✅ Stamina/health cost card tiers in all disciplines
- ✅ GET /actions/possible endpoint
- ✅ Test consolidation: removed overlapping tests and all /tests/* endpoints
- ✅ CI coverage threshold 85% → 80%
- ✅ Renamed discipline_tags → valid_discipline_types
- ✅ ResearchProject cleanup (removed discipline, tier_count fields)

---

## Remaining suggestions for vision.md

### Research encounter design gaps

- **Researched cards are always Attack type** regardless of discipline. The vision implies discipline-appropriate card kinds (Defence, Resource, Mining, etc.). Either document this simplification or add a roadmap item for discipline-to-card-kind mapping in research.
- **Research encounter accessibility**: The Research encounter card starts with `deck: 1, hand: 0`, requiring ~19 encounters before it appears in hand. Consider moving to `hand: 1` so it's accessible from game start, or document the late-game appearance as intentional progression.

### Economy and balancing

- **Insight economy scarcity**: Research tier 1 costs ~30 per-discipline Insight total (10 choose + 20 progress). With only 2 Insight cards per deck generating 1-5 Insight per play, completing a research project requires many encounters. Consider whether current Insight generation rates match the intended progression speed.
- Consider documenting the three-action-within-one-encounter pattern (swap/craft/durability in Crafting) as a design template for future complex encounters.
- Consider mentioning Crafting as the primary "economy sink" for gathered materials (Ore, Plant, Lumber, Fish).

### Architecture documentation

- Document the dual-effect model on gathering cards: domain-specific effects (gains/costs/reductions) plus generic ConcreteEffects (Insight, potentially future types). This cross-cutting pattern enables future effects without modifying domain structs.

---

## Remaining suggestions for roadmap.md

### High priority

- **Discipline-to-card-kind mapping in research**: Researched cards should produce the appropriate CardKind based on the discipline being researched, not always Attack.
- **Register Insight cards for gathering disciplines**: The infrastructure for Insight in all disciplines is in place, but no gathering discipline cards currently carry Insight effects. Card registration changes needed to activate the feature in gameplay.

### Medium priority

- **Crafting card variety**: Add multiple crafting encounter tiers, tune material costs and enemy deck strength.
- **Scouting preview system**: The EnemyCardEffect system now covers all encounter types, enabling scouting previews. A roadmap step for scouting UI/API that leverages effect references would add strategic depth.
- **Enemy card effect balancing**: Now that all enemy types have registered effects, a balancing pass could ensure effects create meaningful encounters across all disciplines.
- **PlantCard and FishCard ConcreteEffect migration**: PlantCard.characteristics and FishCard.value were kept as unique mechanics. Consider whether these could eventually migrate to ConcreteEffect or if they should remain as domain-specific fields permanently.

### Low priority / future

- **Card modification/enhancement in crafting**: Only creation (copy) is implemented. Modification/enhancement is part of the crafting vision but not yet implemented.
- **Research completion tests**: Add scenario tests that verify full research completion once the Insight economy supports it.
- **EnemyCardEffect data-driven configuration**: If the number of effects grows significantly, consider moving from code-based registration to data-driven configuration files.

---

## Implementation notes (historical reference)

### Crafting cost formula
`base_cost = total_power * (1 + num_effects) / 4`, distributed randomly across 2–4 material tokens (max 75% per token) using Fisher-Yates shuffle with seeded RNG.

### Token cost for starting a craft
`min(total_material_cost/100 + 1, remaining_tokens)` with a floor of 2 tokens. Higher-quality cards consume more crafting tokens to start.

### Crafting card swap interpretation
"Replace a card between deck/discard pile and library" — implemented as a bidirectional swap: one card moves FROM deck/discard TO library, and another card moves FROM library TO deck.
