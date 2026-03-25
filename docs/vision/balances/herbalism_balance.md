# Herbalism Balance

This document contains herbalism-specific balancing information. It is the authoritative reference for herbalism balance targets, mechanics, and tuning guidance.

## Target Metrics

Herbalism balance is measured by **yield per durability** — how many Plant tokens a player earns for the HerbalismDurability spent across encounters. Unlike combat (which uses win streaks), gathering disciplines focus on resource efficiency over a session of encounters.

### Strategy Hierarchy (yield per durability)

| Strategy | Description |
|----------|-------------|
| Random | Plays any available herbalism card without considering plant characteristics |
| Greedy | Plays the broadest match card to eliminate the most plants per play |
| Conservative | Plays narrow (single-characteristic) cards for safe, predictable elimination |
| Tactician | Reads remaining plant characteristics, selects optimal match mode and targets to reach exactly 1 plant remaining with minimal durability spent |

- Simple strategies (Random, Greedy, Conservative) should all produce somewhat similar yield-per-durability ratios.
- Tactician strategies should achieve measurably higher yield per durability — reading the plant composition and choosing precise eliminations should be rewarded.
- The gap must be meaningful: tactical play (And, MostCommon, LeastCommon modes) should reliably outperform simple Or-only play.

### Cross-Discipline Yield Parity

Herbalism should produce roughly the same yield value per durability as other gathering disciplines (mining, woodcutting, fishing), even though the yield type (Plant) differs. A discipline may use more encounters than another, as long as the average yield per durability is comparable.

## Herbalism Mechanics

### Core Mechanic: Characteristic Matching

The encounter starts with 50 plant cards (5 types × 10), each with a set of 5 possible characteristics: Fragile, Thorny, Aromatic, Bitter, Luminous.

The goal is to eliminate plants until exactly **1 plant remains** — that is a win, granting 500 Plant tokens.

### Match Modes

Player cards use different match modes to eliminate plants:

| Match Mode | Effect | Risk/Reward |
|-----------|--------|-------------|
| **Or** | Remove plants with ANY of the target characteristics | Broad — removes many plants, risk of over-elimination |
| **And** | Remove plants with ALL target characteristics | Precise — removes fewer plants, lower risk of over-elimination |
| **MostCommon** | Find the most common characteristic, remove matching plants | Context-dependent — effect depends on current plant composition |
| **LeastCommon** | Find the rarest characteristic, remove matching plants | Precise — targets small groups, good for fine-tuning |

### Win and Loss Conditions

- **Win**: Exactly 1 plant remains → 500 Plant tokens
- **Loss**: 0 plants remain (over-elimination), HerbalismDurability ≤ 0, or all hand cards unpayable

### Strategic Tension

The core tension is **precision vs efficiency**: broad matches (Or with multiple characteristics) eliminate many plants quickly but risk over-shooting to 0. Narrow matches (And, single-characteristic Or) are safer but cost more durability per elimination. Tactical players read the plant composition and choose the right mode to minimize total durability spent while reaching exactly 1 plant.

## Token Lifecycle in Herbalism

- **HerbalismDurability**: `PersistentCounter` (initialized at 10,000). Decreased by post-play costs (50–100% of card cost range). Triggers encounter loss if ≤ 0. Persists across encounters — total durability is the session budget.
- **Stamina**: `PersistentCounter`. Pre-play cost on complex match cards (100–150%). Persists across encounters; main recovery comes from resting.
- **Health**: `PersistentCounter`. Pre-play cost on high-tier cards (150–200%). Rare but significant. Persists across encounters.

## Card Composition

~50 player cards across 8 match variations plus stamina recovery:

| Card Type | Match Mode | Characteristics | Cost Profile |
|-----------|-----------|----------------|-------------|
| Simple Or (single) | Or | 1 characteristic | Low durability |
| Simple Or (dual) | Or | 2 characteristics | Low durability |
| And (dual) | And | 2 characteristics | Medium durability + stamina |
| And (triple) | And | 3 characteristics | Medium durability + stamina |
| MostCommon | MostCommon | Context-dependent | Medium durability + stamina |
| LeastCommon | LeastCommon | Context-dependent | Medium durability + stamina |
| High-tier | Various | Multiple | High durability + health |
| Stamina recovery | N/A | Gains stamina | Low durability |

## Config Parameters

Key herbalism config parameters in `configurations/herbalism/cards.json`:
- Plant deck composition (types, counts, characteristic distributions)
- Match mode card counts per type
- Durability cost ranges (min/max per match mode)
- Stamina cost ranges (min/max for complex modes)
- Health cost ranges (min/max for high-tier cards)
- Plant token reward per win (currently 500)

## Tuning Tips

- **Reward is binary**: Unlike mining (variable yield), herbalism has a fixed 500-Plant reward per win. Balance is about the win rate and durability cost per attempt, not reward magnitude.
- **Plant composition drives difficulty**: The distribution of characteristics across plant types determines how easy it is to isolate a single plant. More uniform distributions make precision harder; more varied distributions make it easier.
- **Match mode costs as the balance lever**: The cost differential between Or (cheap, imprecise) and And/MostCommon/LeastCommon (expensive, precise) is the primary lever for tiered balance. If precise modes are too cheap, simple strategies converge with tactical ones.
- **Over-elimination risk**: The main failure mode is eliminating all plants. Cards that remove too many plants per play increase this risk. The Or mode with multiple characteristics is the highest-risk play.
- **Durability budget**: 10,000 durability across all herbalism encounters. Higher per-encounter durability cost means fewer total encounters (and fewer chances to earn Plant tokens).
- **Tiered balance enforcement**: Tactical play (reading characteristics, choosing And/LeastCommon at the right moment) must achieve more wins per durability than random Or-mode play. If strategies converge, increase the cost differential between broad and precise match modes.
- **RNG Coupling**: Changing card counts changes the RNG state for the entire game. Only aggregate metrics across many encounters are meaningful for comparison.
