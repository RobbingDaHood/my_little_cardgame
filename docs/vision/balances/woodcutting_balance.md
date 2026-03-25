# Woodcutting Balance

This document contains woodcutting-specific balancing information. It is the authoritative reference for woodcutting balance targets, mechanics, and tuning guidance.

## Target Metrics

Woodcutting balance is measured by **yield per durability** — how much Lumber a player earns for the WoodcuttingDurability spent across encounters. Unlike combat (which uses win streaks), gathering disciplines focus on resource efficiency over a session of encounters.

### Yield-per-Durability Targets

All yield disciplines (mining, herbalism, woodcutting, fishing) share the same aggregate target: **X–Y yield tokens per Z total durability spent**. These targets are tuned in the balance simulation step (see roadmap B2.6) and should be identical across disciplines to ensure no single gathering path dominates.

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

Woodcutting uses a poker-inspired pattern system. The player plays up to **max_plays** chop cards, each producing a numeric value. After all plays (or early stop), the sequence is evaluated for patterns.

### Chop Types

All chop types share the **same value range** (e.g., 1–10). The number of distinct chop types and the exact range bounds are configuration-driven balance levers. Having all types produce values from the same range means the tactical distinction between chop types comes from their **cost profiles** and **availability**, not inherent value advantages. SplitChop is unlocked via research/crafting (no starting cards).

### Pattern Evaluation

Patterns are evaluated in priority order. The highest-matching pattern determines the multiplier:

| Pattern | Description |
|---------|-------------|
| High Card | No pattern — base reward (lowest multiplier) |
| Pair | Two cards with same value |
| Two Pair | Two different pairs |
| Three of a Kind | Three cards with same value |
| Straight | Consecutive values |
| Flush | All same chop type |
| Full House | Three of a kind + pair |
| Rare combinations | Complex multi-pattern hands (highest multipliers) |

Pattern multipliers are calibrated using **sqrt inverse-probability scaling**: common patterns get low multipliers, rare patterns get significantly higher multipliers. **Rare combinations should have quite good rewards** to motivate risk-taking — the best strategy should NOT be to always play for simple, safe combos. Building toward a rare pattern should be a viable and rewarding strategy when the hand supports it.

### Reward Calculation

```
lumber_reward = base_rewards × pattern_multiplier
```

Where `base_rewards` is a configured Lumber amount. The multiplier from pattern evaluation scales the reward, making pattern-building the core tactical lever.

### Early Stop

The player may stop playing cards before reaching max_plays. This is **not an abort** — the pattern is evaluated with the cards played so far, rewards are granted, and durability costs are only paid for cards actually played. Early stop is a key tactical decision: stop early with a good partial pattern vs risk weakening it with additional cards.

### Durability Depletion

If WoodcuttingDurability reaches 0 during play, the encounter ends immediately as a loss, but **rewards are still granted** — the pattern is evaluated with cards played so far and the scaled reward is applied. This means running out of durability is costly on the record but the player keeps what they earned.

## Token Lifecycle in Woodcutting

- **WoodcuttingDurability**: Persistent counter. Decreased by post-play costs. Triggers encounter end (with rewards, pattern evaluated) if ≤ 0. Persists across encounters — total durability is the session budget. **Note**: The initial durability value is a testing shortcut; after rest encounter balancing, the starting value will likely be significantly lower (closer to one-tenth of the current value).
- **Stamina**: Persistent counter. Pre-play cost on advanced chops. Persists across encounters; main recovery comes from resting.
- **Health**: Persistent counter. Pre-play cost on high-tier cards. Rare but significant. Persists across encounters.

## Config Parameters

Key woodcutting config parameters in `configurations/woodcutting/cards.json`:
- Chop type value range (shared across all types — the number of types and range bounds are balance levers)
- Card counts per chop type and variation
- Durability cost ranges (min/max per chop type)
- Stamina cost ranges (min/max for advanced chops)
- Health cost ranges (min/max for high-tier chops)
- Pattern multiplier table (multiplier per pattern type)
- Base reward (Lumber amount)
- Max plays per encounter

## Tuning Tips

- **Pattern multipliers are the primary balance lever**: The relationship between pattern rarity and multiplier magnitude determines whether tactical play is rewarded. Common patterns (pair, two pair) should have modest multipliers; rare patterns (straight, flush, full house) should have significantly higher multipliers to justify the risk and skill.
- **Rare combos must be rewarding**: Rare combinations should have quite good rewards to motivate risk-taking and creative pattern-building. The best strategy should NOT be to always play for safe, simple combos — players who recognise and build toward rare hands should be meaningfully rewarded.
- **sqrt inverse-probability scaling**: Current multipliers use sqrt of inverse probability. This prevents rare patterns from being disproportionately valuable while still rewarding them meaningfully. Adjusting the scaling function (e.g., log vs sqrt vs linear) changes the reward curve shape.
- **Early stop as tactical lever**: If early stop is too safe (stop after a few cards with a pair = good yield), tactical play loses value. If early stop is too punishing (partial hands always get low multipliers), players are forced to play all cards regardless. The balance point is where stopping mid-encounter with a good pattern is viable but playing more for a better pattern is rewarded.
- **Uniform chop value range**: All chop types produce values from the same range, so pattern probability depends on card selection and cost management, not inherent value tiers. The overlap of a shared range means pairs are achievable from any type; straights require spreading across the range.
- **Durability depletion grants rewards**: Running out of durability still triggers pattern evaluation and reward granting. This makes durability management an efficiency concern (fewer remaining encounters) rather than a catastrophic-loss concern.
- **Durability budget**: Total durability across all woodcutting encounters bounds the session. High-cost cards eat into the durability budget faster, creating tension between playing expensive cards for better patterns vs cheap cards for more encounters.
- **Tiered balance enforcement**: Tactical pattern-building (recognizing when to aim for a straight vs a flush, timing early stop) must produce higher yield per durability than random chop selection. If strategies converge, increase multiplier spread or adjust cost differentials between chop types.
- **RNG Coupling**: Changing card counts changes the RNG state for the entire game. Only aggregate metrics across many encounters are meaningful for comparison.
