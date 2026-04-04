# Woodcutting Balance

This document contains woodcutting-specific balancing information. It is the authoritative reference for woodcutting balance targets, mechanics, and tuning guidance.

## Target Metrics

Woodcutting balance is measured by **yield per durability** — how much Lumber a player earns for the WoodcuttingDurability spent across encounters. Unlike combat (which uses win streaks), gathering disciplines focus on resource efficiency over a session of encounters.

### Yield-per-Durability Targets

All yield disciplines (mining, herbalism, woodcutting, fishing) share the same aggregate target: **0.5–4.0 yield per durability**. This range is deliberately wide to give room for balancing while maintaining cross-discipline parity.

#### Tier-Based Targets

| Tier | Yield/Durability Range | Description |
|------|----------------------|-------------|
| **Tier 1** (Simple) | 0.5 – 2.0 | Random, Greedy, Conservative — no encounter-state awareness |
| **Tier 2** (Tactical) | 1.5 – 4.0 | Tactician variants — encounter-aware, exploits discipline mechanics |

The overlap between tiers (1.5–2.0) is intentional — a well-tuned simple strategy may approach the lower end of the tactical range, but tactical strategies should consistently land higher. These targets are tuned in the balance simulation step (see roadmap) and must be comparable across disciplines to enable parallel balancing — if one discipline significantly over- or under-produces relative to this band, its config needs adjustment.

> **Note**: This document contains only balance goals and general tips on how to achieve them. Simulation results belong in PR descriptions and commit messages, not here.

### Strategy Hierarchy (yield per durability)

| Strategy | Tier | Description |
|----------|------|-------------|
| Random | 1 (Simple) | Plays any available woodcutting card without considering pattern potential |
| Greedy | 1 (Simple) | Always plays the highest-value chop card available |
| Conservative | 1 (Simple) | Plays lowest-cost cards to preserve durability |
| PatternBuilder | 2 (Tactical) | Reads cards played so far, builds toward high-value patterns (same chop type) |
| DurabilityConserver | 2 (Tactical) | Encounter-state-aware durability management — picks lowest durability-cost cards, concludes early when budget is tight. Wins through resource conservation, not pattern optimization. |

- Simple strategies (Random, Greedy, Conservative) should all produce somewhat similar yield-per-durability ratios within the tier 1 range (0.5–2.0).
- Tactician strategies should achieve measurably higher yield per durability, landing in the tier 2 range (1.5–4.0) — pattern-building and early-stop timing must be rewarded.
- The gap must reflect the skill involved in recognizing pattern potential and deciding when to stop.

### Tier-2 Strategy Requirements

There must be at least **2 distinct tier-2 runners** that outperform all tier-1 strategies:

1. **Yield-optimizer**: A tactician that exploits pattern multipliers to maximize Lumber reward — recognizes and builds toward high-multiplier patterns, uses early stop optimally.
2. **Non-yield tactician**: A tactician that beats tier-1 runners **without ever adjusting yield outcome** — it must never rely on pattern-multiplier optimization. Instead, it wins through superior durability conservation, cost management, and card selection (playing the cheapest viable cards to stretch the durability budget across more encounters).

Choosing the correct encounter via scouting is a valid additional tier-2 tactic, but it does **not** count toward the "non-yield tactician" requirement. There must always be at least one tier-2 runner that beats tier-1 purely through in-encounter play decisions.

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

If WoodcuttingDurability reaches 0 during play, the encounter ends immediately as a loss with no rewards granted.

## Token Lifecycle in Woodcutting

- **WoodcuttingDurability**: Persistent counter. Decreased by post-play costs. Triggers encounter loss if ≤ 0. Persists across encounters — total durability is the session budget. **Note**: The initial durability value is a testing shortcut; after rest encounter balancing, the starting value will likely be significantly lower (closer to one-tenth of the current value).
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
- **Durability budget**: Total durability across all woodcutting encounters bounds the session. High-cost cards eat into the durability budget faster, creating tension between playing expensive cards for better patterns vs cheap cards for more encounters.
- **Tiered balance enforcement**: Tactical pattern-building (recognizing when to aim for a straight vs a flush, timing early stop) must produce higher yield per durability than random chop selection. If strategies converge, increase multiplier spread or adjust cost differentials between chop types.
- **RNG Coupling**: Changing card counts changes the RNG state for the entire game. Only aggregate metrics across many encounters are meaningful for comparison.

### Insights from B2.6 Balance Tuning

These insights were discovered during the B2.6 woodcutting balance tuning (PR #70) and should guide future tuning attempts:

- **max_plays must stay at 8**: With 5 starting hand cards and auto-draw, playing 8 cards means 5 cards are known up front and 3 are drawn during play. This creates strategic depth — players can plan around visible cards while gambling on draws. Reducing max_plays (e.g., to 4) eliminates the unknowns and flattens strategic differentiation.
- **Auto-draw eliminates card depletion**: Every card play (including the final play of an encounter) draws a replacement. With 8 plays and 8 draws per encounter, the net card change is 0. Hands persist between encounters at full size, meaning **all encounters throughout the session are active**. Durability — not card supply — is the true session limiter.
- **Durability conservation is a valid tier-2 tactic**: The DurabilityConserver strategy proves that a non-yield tactician can beat tier-1 by picking lowest-cost cards and concluding early when the durability budget is tight. This satisfies the tier-2 requirement without any pattern optimization.
- **Cost profile differentiation drives strategy**: With auto-draw maintaining hand size, the key strategic dimension is cost management — which cards to play when, and when to stop. Lowering absolute cost ranges (e.g., durability 12–35 instead of 50–100) increases the number of plays before durability runs out, amplifying the difference between cheap and expensive cards.

## General Design Principles

These principles apply across all disciplines. For full details, see `docs/design/vision.md`.

### Card Cost Distribution

- **Free cards** (no cost) should be the most common card type in every deck.
- **Stamina-cost cards** should be moderately common and always outperform free cards in raw effect value.
- **Health-cost cards** should be rare but powerful, always outperforming stamina-cost cards.
- This creates a risk/reward spectrum: safe low-output plays → moderate-cost moderate-output → high-risk high-output.

### Mutator Scope

Balance mutators (the agents implementing balance changes) **may** change within their discipline:
- Any CardEffect within woodcutting (including suggesting new CardEffects as a last resort)
- Any Card within woodcutting (including suggesting new Cards, but try without first)
- Any encounter within woodcutting (including suggesting new encounters, but try without first)

Balance mutators **must NOT** change:
- Starting Health, Stamina, or any player starting tokens
- Health or Stamina after death
- Hand sizes (all must remain 5)
- Deck sizes (all must remain 50)
- Anything outside the woodcutting discipline

### Deck and Hand Sizing

- All player deck hand sizes: **5** (controlled by per-deck MaxHand tokens)
- All player deck sizes: **50** (controlled by per-deck MaxDeck tokens)
- Do NOT change deck or hand sizes to fix balance issues — adjust card effects and encounter parameters instead.
