# Herbalism Balance

This document contains herbalism-specific balancing information. It is the authoritative reference for herbalism balance targets, mechanics, and tuning guidance.

## Target Metrics

Herbalism balance is measured by **yield per durability** — how many Plant tokens a player earns for the HerbalismDurability spent across encounters. Unlike combat (which uses win streaks), gathering disciplines focus on resource efficiency over a session of encounters.

### Yield-per-Durability Targets

All yield disciplines (mining, herbalism, woodcutting, fishing) share the same aggregate target: **0.5–4.0 yield per durability**. This range is deliberately wide to give room for balancing while maintaining cross-discipline parity.

#### Tier-Based Targets

| Tier | Yield/Durability Range | Description |
|------|----------------------|-------------|
| **Tier 1** (Simple) | 0.5 – 2.0 | Random, Greedy, Conservative — no encounter-state awareness |
| **Tier 2** (Tactical) | 1.5 – 4.0 | Tactician variants — encounter-aware, exploits discipline mechanics |

The overlap between tiers (1.5–2.0) is intentional — a well-tuned simple strategy may approach the lower end of the tactical range, but tactical strategies should consistently land higher. These targets are tuned in the balance simulation step (see roadmap) and must be comparable across disciplines to enable parallel balancing — if one discipline significantly over- or under-produces relative to this band, its config needs adjustment.

### Strategy Hierarchy (yield per durability)

| Strategy | Tier | Description |
|----------|------|-------------|
| Random | 1 (Simple) | Plays any available herbalism card without considering plant characteristics |
| Greedy | 1 (Simple) | Plays the broadest match card to eliminate the most plants per play |
| Conservative | 1 (Simple) | Plays narrow (single-characteristic) cards for safe, predictable elimination |
| Tactician | 2 (Tactical) | Reads remaining plant characteristics, selects optimal match mode and targets to reach exactly 1 plant remaining with minimal durability spent |

- Simple strategies (Random, Greedy, Conservative) should all produce somewhat similar yield-per-durability ratios within the tier 1 range (0.5–2.0).
- Tactician strategies should achieve measurably higher yield per durability, landing in the tier 2 range (1.5–4.0) — reading the plant composition and choosing precise eliminations should be rewarded.
- The gap must be meaningful: tactical play (And, MostCommon, LeastCommon modes) should reliably outperform simple Or-only play.

### Tier-2 Strategy Requirements

There must be at least **2 distinct tier-2 runners** that outperform all tier-1 strategies:

1. **Yield-optimizer**: A tactician that exploits yield-related card effects or optimal elimination sequences to maximize Plant reward per encounter.
2. **Non-yield tactician**: A tactician that beats tier-1 runners **without ever adjusting yield outcome** — it must never rely on yield-boosting effects. Instead, it wins through superior match-mode selection, durability conservation, and precision elimination (reaching exactly 1 plant with fewer costly plays).

Choosing the correct encounter via scouting is a valid additional tier-2 tactic, but it does **not** count toward the "non-yield tactician" requirement. There must always be at least one tier-2 runner that beats tier-1 purely through in-encounter play decisions.

### Cross-Discipline Yield Parity

Herbalism should produce roughly the same yield value per durability as other gathering disciplines (mining, woodcutting, fishing), even though the yield type (Plant) differs. A discipline may use more encounters than another, as long as the average yield per durability is comparable.

## Herbalism Mechanics

### Core Mechanic: Characteristic Matching

The encounter starts with a set of plant cards across several types, each with a set of possible characteristics (e.g., Fragile, Thorny, Aromatic, Bitter, Luminous).

The goal is to eliminate plants until exactly **1 plant remains** — that is a win, granting the base Plant token reward.

### Match Modes

Player cards use different match modes to eliminate plants:

| Match Mode | Effect | Risk/Reward |
|-----------|--------|-------------|
| **Or** | Remove plants with ANY of the target characteristics | Broad — removes many plants, risk of over-elimination |
| **And** | Remove plants with ALL target characteristics | Precise — removes fewer plants, lower risk of over-elimination |
| **MostCommon** | Find the most common characteristic, remove matching plants | Context-dependent — effect depends on current plant composition |
| **LeastCommon** | Find the rarest characteristic, remove matching plants | Precise — targets small groups, good for fine-tuning |

### Win and Loss Conditions

- **Win**: Exactly 1 plant remains → base Plant token reward
- **Loss**: 0 plants remain (over-elimination), HerbalismDurability ≤ 0, or all hand cards unpayable

### Strategic Tension

The core tension is **precision vs efficiency**: broad matches (Or with multiple characteristics) eliminate many plants quickly but risk over-shooting to 0. Narrow matches (And, single-characteristic Or) are safer but cost more durability per elimination. Tactical players read the plant composition and choose the right mode to minimize total durability spent while reaching exactly 1 plant.

## Token Lifecycle in Herbalism

- **HerbalismDurability**: Persistent counter. Decreased by post-play costs. Triggers encounter loss if ≤ 0. Persists across encounters — total durability is the session budget. **Note**: The initial durability value is a testing shortcut; after rest encounter balancing, the starting value will likely be significantly lower (closer to one-tenth of the current value).
- **Stamina**: Persistent counter. Pre-play cost on complex match cards. Persists across encounters; main recovery comes from resting.
- **Health**: Persistent counter. Pre-play cost on high-tier cards. Rare but significant. Persists across encounters.

## Card Composition

Player cards span several match mode variations plus stamina recovery. The cost profile increases with match complexity: simple Or cards are cheap, And/MostCommon/LeastCommon cost more, and high-tier multi-mode cards add health costs. The exact composition is configuration-driven — see `configurations/herbalism/cards.json`.

## Config Parameters

Key herbalism config parameters in `configurations/herbalism/cards.json`:
- Plant deck composition (types, counts, characteristic distributions)
- Match mode card counts per type
- Durability cost ranges (min/max per match mode)
- Stamina cost ranges (min/max for complex modes)
- Health cost ranges (min/max for high-tier cards)
- Base Plant token reward per win

## Tuning Tips

- **Reward is binary**: Unlike mining (variable yield), herbalism has a fixed reward per win. Balance is about the win rate and durability cost per attempt, not reward magnitude.
- **Plant composition drives difficulty**: The distribution of characteristics across plant types determines how easy it is to isolate a single plant. More uniform distributions make precision harder; more varied distributions make it easier.
- **Match mode costs as the balance lever**: The cost differential between Or (cheap, imprecise) and And/MostCommon/LeastCommon (expensive, precise) is the primary lever for tiered balance. If precise modes are too cheap, simple strategies converge with tactical ones.
- **Over-elimination risk**: The main failure mode is eliminating all plants. Cards that remove too many plants per play increase this risk. The Or mode with multiple characteristics is the highest-risk play.
- **Durability budget**: Total durability across all herbalism encounters bounds the session. Higher per-encounter durability cost means fewer total encounters (and fewer chances to earn Plant tokens).
- **Tiered balance enforcement**: Tactical play (reading characteristics, choosing And/LeastCommon at the right moment) must achieve more wins per durability than random Or-mode play. If strategies converge, increase the cost differential between broad and precise match modes.
- **RNG Coupling**: Changing card counts changes the RNG state for the entire game. Only aggregate metrics across many encounters are meaningful for comparison.

## General Design Principles

These principles apply across all disciplines. For full details, see `docs/design/vision.md`.

### Card Cost Distribution

- **Free cards** (no cost) should be the most common card type in every deck.
- **Stamina-cost cards** should be moderately common and always outperform free cards in raw effect value.
- **Health-cost cards** should be rare but powerful, always outperforming stamina-cost cards.
- This creates a risk/reward spectrum: safe low-output plays → moderate-cost moderate-output → high-risk high-output.

### Mutator Scope

Balance mutators (the agents implementing balance changes) **may** change within their discipline:
- Any CardEffect within herbalism (including suggesting new CardEffects as a last resort)
- Any Card within herbalism (including suggesting new Cards, but try without first)
- Any encounter within herbalism (including suggesting new encounters, but try without first)

Balance mutators **must NOT** change:
- Starting Health, Stamina, or any player starting tokens
- Health or Stamina after death
- Hand sizes (all must remain 5)
- Deck sizes (all must remain 50)
- Anything outside the herbalism discipline

### Deck and Hand Sizing

- All player deck hand sizes: **5** (controlled by per-deck MaxHand tokens)
- All player deck sizes: **50** (controlled by per-deck MaxDeck tokens)
- Do NOT change deck or hand sizes to fix balance issues — adjust card effects and encounter parameters instead.
