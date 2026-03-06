# Vision & Roadmap Suggestions — Crafting (9.6) Implementation

## What was implemented

- **Full crafting encounter lifecycle**: start → swap/craft/durability actions → conclude/abort → scouting
- **Three crafting actions**: Card swap (1 token), craft-a-card mini-game (variable tokens), add durability (1 token + materials)
- **Craft-a-card mini-game**: Turn-based card game where player and enemy crafting cards modify material costs; costs floored at 50% of original per token type
- **Crafting cost field**: Every player card gets a `crafting_cost: HashMap<TokenType, i64>` computed at creation, spread across Ore/Plant/Lumber/Fish
- **6 player crafting cards** + **1 crafting encounter card** with 4 enemy card types
- **CraftingToken & CraftingMaxHand** token types for encounter-scoped economy
- **Abort always PlayerWon** — no penalty for early exit
- **7 scenario integration tests** covering all crafting flows

## What was deferred / not yet implemented

- **Stamina/Health cost crafting cards**: The roadmap says "Stamina and Health tokens should be usable in CardEffects with costs within crafting" — the current implementation uses CraftingToken as the only cost for crafting cards. A future pass should add some crafting cards that cost Stamina or Health for stronger cost-reduction effects.
- **Card modification/enhancement**: The roadmap mentions "create, modify, and enhance cards" but only creation (copy) is implemented. Modification and enhancement could be future crafting sub-actions.
- **Scaling crafting encounters**: Only one crafting encounter definition exists. Future work could add encounter variants with different enemy deck compositions, initial token amounts, or material cost multipliers.

## Suggested roadmap.md updates

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
