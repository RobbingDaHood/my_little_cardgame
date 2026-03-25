# Woodcutting Balance

This document contains woodcutting-specific balancing information. It is the authoritative reference for woodcutting balance targets, mechanics, and tuning guidance.

## Target Metrics

Woodcutting balance is measured by **yield per durability** — how much Lumber a player earns for the WoodcuttingDurability spent across encounters. Unlike combat (which uses win streaks), gathering disciplines focus on resource efficiency over a session of encounters.

### Strategy Hierarchy (yield per durability)

| Strategy | Description |
|----------|-------------|
| Random | Plays any available woodcutting card without considering pattern potential |
| Greedy | Always plays the highest-value chop card available |
| Conservative | Plays lowest-cost cards to preserve durability |
| Tactician | Reads cards played so far, builds toward high-value patterns, times early stop optimally |

- Simple strategies (Random, Greedy, Conservative) should all produce somewhat similar yield-per-durability ratios.
- Tactician strategies should achieve measurably higher yield per durability — pattern-building and early-stop timing must be rewarded.
- The gap must reflect the skill involved in recognizing pattern potential and deciding when to stop.

### Cross-Discipline Yield Parity

Woodcutting should produce roughly the same yield value per durability as other gathering disciplines (mining, herbalism, fishing), even though the yield type (Lumber) differs. A discipline may use more encounters than another, as long as the average yield per durability is comparable.

## Woodcutting Mechanics

### Core Mechanic: Pattern Matching

Woodcutting uses a poker-inspired pattern system. The player plays up to **max_plays** (default 8) chop cards, each producing a numeric value. After all plays (or early stop), the sequence is evaluated for patterns.

### Chop Types

| Chop Type | Value Range | Description |
|-----------|------------|-------------|
| LightChop | 1–3 | Low values, cheap |
| MediumChop | 3–6 | Mid-range values |
| HeavyChop | 3–7 | Wider range, more costly |
| PrecisionChop | 7–9 | High values, expensive |
| SplitChop | 4–8 | Unlocked via research/crafting (no starting cards) |

### Pattern Evaluation

Patterns are evaluated in priority order. The highest-matching pattern determines the multiplier:

| Pattern | Multiplier Range | Description |
|---------|-----------------|-------------|
| High Card | 1.0x | No pattern — base reward |
| Pair | ~1.0–1.5x | Two cards with same value |
| Two Pair | ~1.5–2.0x | Two different pairs |
| Three of a Kind | ~2.0–2.5x | Three cards with same value |
| Straight | ~2.5–3.5x | Consecutive values |
| Flush | ~2.0–3.0x | All same chop type |
| Full House | ~3.0–4.0x | Three of a kind + pair |
| Rare combinations | Up to 5.0x+ | Complex multi-pattern hands |

Pattern multipliers are calibrated using **sqrt inverse-probability scaling**: most common patterns get low multipliers (1.0–1.5x), rare patterns get significantly higher multipliers (up to 55.0x for extremely rare combinations).

### Reward Calculation

```
lumber_reward = base_rewards × pattern_multiplier
```

Where `base_rewards` defaults to 1000 Lumber. A high-card hand yields 1000; a full house might yield 3500.

### Early Stop

The player may stop playing cards before reaching max_plays. This is **not an abort** — the pattern is evaluated with the cards played so far, rewards are granted, and durability costs are only paid for cards actually played. Early stop is a key tactical decision: stop early with a good partial pattern vs risk weakening it with additional cards.

## Token Lifecycle in Woodcutting

- **WoodcuttingDurability**: `PersistentCounter` (initialized at 10,000). Decreased by post-play costs (50–100% of card cost range). Triggers encounter loss if ≤ 0. Persists across encounters — total durability is the session budget.
- **Stamina**: `PersistentCounter`. Pre-play cost on advanced chops (100–250%). Persists across encounters; main recovery comes from resting.
- **Health**: `PersistentCounter`. Pre-play cost on high-tier cards (150–200%). Rare but significant. Persists across encounters.

## Config Parameters

Key woodcutting config parameters in `configurations/woodcutting/cards.json`:
- Chop type value ranges (min/max per chop type)
- Card counts per chop type and variation
- Durability cost ranges (min/max per chop type)
- Stamina cost ranges (min/max for advanced chops)
- Health cost ranges (min/max for high-tier chops)
- Pattern multiplier table (multiplier per pattern type)
- Base reward (default 1000 Lumber)
- Max plays per encounter (default 8)

## Tuning Tips

- **Pattern multipliers are the primary balance lever**: The relationship between pattern rarity and multiplier magnitude determines whether tactical play is rewarded. Multipliers should scale with rarity — common patterns (pair, two pair) should have modest multipliers; rare patterns (straight, flush, full house) should have significantly higher multipliers to justify the risk and skill.
- **sqrt inverse-probability scaling**: Current multipliers use sqrt of inverse probability. This prevents rare patterns from being disproportionately valuable while still rewarding them meaningfully. Adjusting the scaling function (e.g., log vs sqrt vs linear) changes the reward curve shape.
- **Early stop as tactical lever**: If early stop is too safe (stop after 2–3 cards with a pair = good yield), tactical play loses value. If early stop is too punishing (partial hands always get low multipliers), players are forced to play all 8 cards regardless. The balance point is where stopping at 4–6 cards with a good pattern is viable but playing all 8 for a better pattern is rewarded.
- **Chop type value ranges create pattern probability**: The overlap between chop type ranges (e.g., MediumChop 3–6 overlaps with HeavyChop 3–7) determines how likely pairs and straights are. More overlap = more pairs; less overlap = more straights.
- **Durability budget**: 10,000 durability across all woodcutting encounters. High-cost cards (PrecisionChop, HeavyChop) eat into the durability budget faster, creating tension between playing expensive cards for better patterns vs cheap cards for more encounters.
- **Tiered balance enforcement**: Tactical pattern-building (recognizing when to aim for a straight vs a flush, timing early stop) must produce higher yield per durability than random chop selection. If strategies converge, increase multiplier spread or adjust cost differentials between chop types.
- **RNG Coupling**: Changing card counts changes the RNG state for the entire game. Only aggregate metrics across many encounters are meaningful for comparison.
