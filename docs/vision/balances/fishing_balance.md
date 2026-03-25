# Fishing Balance

This document contains fishing-specific balancing information. It is the authoritative reference for fishing balance targets, mechanics, and tuning guidance.

## Target Metrics

Fishing balance is measured by **yield per durability** — how many Fish tokens a player earns for the FishingDurability spent across encounters. Unlike combat (which uses win streaks), gathering disciplines focus on resource efficiency over a session of encounters.

### Strategy Hierarchy (yield per durability)

| Strategy | Description |
|----------|-------------|
| Random | Plays any available fishing card without considering fish value or range |
| Greedy | Always plays the highest-value fishing card available |
| Conservative | Plays lowest-cost cards to preserve durability |
| Tactician | Reads fish deck distribution, manages valid range, selects values that maximize wins within the current range window |

- Simple strategies (Random, Greedy, Conservative) should all produce somewhat similar yield-per-durability ratios.
- Tactician strategies should achieve measurably higher yield per durability — range management and value selection must be rewarded.
- The gap must reflect the skill of reading the fish distribution and selecting optimal values.

### Cross-Discipline Yield Parity

Fishing should produce roughly the same yield value per durability as other gathering disciplines (mining, herbalism, woodcutting), even though the yield type (Fish) differs. A discipline may use more encounters than another, as long as the average yield per durability is comparable.

## Fishing Mechanics

### Core Mechanic: Numeric Duel

Fishing is a numeric duel between the player and a fish deck. Each round:

```
result = max(0, player_value - fish_value)
```

The round is a **win** if `result ∈ [FishingRangeMin, FishingRangeMax]`.

This creates a "sweet spot" mechanic: the player must play a value that is higher than the fish's value, but not by too much — the difference must fall within the valid range.

### Win and Loss Conditions

- **Win**: `turns_won ≥ wins_required` (default 4) within `max_turns` (default 8) → 1000 Fish tokens
- **Loss**: Max turns exhausted without enough wins, FishingDurability ≤ 0, or all hand cards unpayable

### Encounter-Scoped Tokens

| Token | Initial Value | Description |
|-------|--------------|-------------|
| FishingRangeMin | 100 | Lower bound of valid range |
| FishingRangeMax | 300 | Upper bound of valid range |
| FishAmount | 1 | Wins counted per successful turn |

### Smart Value Selection

When multiple fishing values are available, the optimal strategy:
1. Find values where `(player_value - fish_value) ∈ [RangeMin, RangeMax]`
2. Among qualifying values, pick the one closest to range center
3. Fall back to first value if none qualify

## Fish Deck Composition

50 fish cards with weighted distribution:

| Fish Value | Count | Proportion | Balance Role |
|-----------|-------|-----------|-------------|
| 100 | 16 | 32% | Easy targets — most player values will land in range |
| 300 | 17 | 34% | Medium targets — require mid-range player values |
| 500 | 11 | 22% | Hard targets — need high player values, risk overshooting low fish |
| 700 | 6 | 12% | Very hard — only highest player values can land in range |

The distribution is weighted toward lower-value fish, creating a base win rate, with occasional high-value fish that test range management.

## Token Lifecycle in Fishing

- **FishingDurability**: `PersistentCounter` (initialized at 10,000). Decreased by post-play costs (30–60% of card cost range). Triggers encounter loss if ≤ 0. Persists across encounters — total durability is the session budget.
- **Stamina**: `PersistentCounter`. Pre-play cost on advanced cards (50–100%). Persists across encounters; main recovery comes from resting.
- **Health**: `PersistentCounter`. Pre-play cost on high-tier cards (60–100%). Persists across encounters.
- **FishingRangeMin/FishingRangeMax**: Encounter-scoped. Modified by range-modifier effects on player cards. Min can shift -150 to +50; Max can shift -50 to +150. Resets each encounter.
- **FishAmount**: Encounter-scoped. Modified by amount-modifier effects (±1). Affects how many wins count per successful turn. Resets each encounter.

## Player Card Tiers

| Tier | Value Range | Cost Profile | Range/Amount Modifiers |
|------|------------|-------------|----------------------|
| Basic | 50–200 | Low durability (30–60%) | None or minor range adjustments |
| Mid | 250–450 | Medium durability + stamina (50–100%) | May include range widening |
| High | 500–750 | High durability + health (60–100%) | May include range narrowing or FishAmount boost |

## Config Parameters

Key fishing config parameters in `configurations/fishing/cards.json`:
- Player card value ranges (min/max per tier)
- Fish deck value distribution (count per value level)
- Wins required per encounter (default 4)
- Max turns per encounter (default 8)
- Initial valid range (FishingRangeMin, FishingRangeMax)
- Range modifier limits (min/max shifts)
- FishAmount modifier limits
- Durability cost ranges (min/max per tier)
- Stamina cost ranges (min/max per tier)
- Health cost ranges (min/max per tier)
- Fish token reward per win (currently 1000)

## Tuning Tips

- **Valid range width is the primary balance lever**: A wider range (larger RangeMax - RangeMin) makes it easier to land wins; a narrower range increases difficulty. The initial range (100–300) means the player needs a difference of 100–300 between their value and the fish's value.
- **Fish deck distribution determines base difficulty**: More low-value fish (100, 300) makes the encounter easier because more player cards will land in range. More high-value fish (500, 700) forces players to use expensive high-tier cards.
- **Range modifiers as tactical lever**: Cards that widen the valid range (increase RangeMax, decrease RangeMin) make subsequent turns easier. This creates a tactical dimension: spend a turn widening the range vs immediately going for wins.
- **FishAmount as multiplier**: Cards that boost FishAmount allow double-counting wins on a turn, creating "power turns." Balance FishAmount modifiers carefully — too accessible and they trivialize the encounter.
- **Dual loss conditions**: Both turn exhaustion and durability depletion can end an encounter. The balance between max_turns and durability cost per turn determines which is the binding constraint. Currently max_turns=8 and wins_required=4 means a 50% win rate per turn is needed.
- **Durability budget**: 10,000 durability across all fishing encounters. Lower per-card durability cost (30–60%) compared to other disciplines means fishing supports more encounters but each encounter's yield is also high (1000 Fish per win).
- **Tiered balance enforcement**: Tactical range management (widening range on early turns, selecting optimal values based on expected fish distribution) must produce more wins per durability than random value selection. If strategies converge, increase the impact of range modifiers or add cards with stronger range/amount trade-offs.
- **RNG Coupling**: Changing card counts changes the RNG state for the entire game. Only aggregate metrics across many encounters are meaningful for comparison.
