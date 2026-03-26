# Fishing Balance

This document contains fishing-specific balancing information. It is the authoritative reference for fishing balance targets, mechanics, and tuning guidance.

## Target Metrics

Fishing balance is measured by **yield per durability** — how many Fish tokens a player earns for the FishingDurability spent across encounters. Unlike combat (which uses win streaks), gathering disciplines focus on resource efficiency over a session of encounters.

### Yield-per-Durability Targets

All yield disciplines (mining, herbalism, woodcutting, fishing) share the same aggregate target: **2,000–4,000 yield tokens per 10,000 total durability spent** (0.2–0.4 yield per durability). The tactician strategy should reliably land in the upper half of this range, while simple strategies land in the lower half. These targets are tuned in the balance simulation step (see roadmap B2.7) and must be identical across disciplines to enable parallel balancing — if one discipline significantly over- or under-produces relative to this band, its config needs adjustment.

### Strategy Hierarchy (yield per durability)

| Strategy | Description |
|----------|-------------|
| Random | Plays any available fishing card without considering fish value or range |
| Greedy | Always plays the highest-value fishing card available |
| Conservative | Plays lowest-cost cards to preserve durability |
| Tactician | Manages valid range (widening/narrowing), selects values that best match the current fish, and boosts FishAmount for reward scaling |

- Simple strategies (Random, Greedy, Conservative) should all produce somewhat similar yield-per-durability ratios.
- Tactician strategies should achieve measurably higher yield per durability — range management, best-matching value selection, and FishAmount optimization must all be rewarded.
- The gap must reflect the skill of reading the fish distribution and combining multiple optimisation levers.

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

### Smart Value Selection (Best Matching)

Fishing cards can have **multiple values**. When a card is played, the system automatically selects the best-matching value — the one most likely to produce a result within the valid range given the current fish value. This is a core mechanic, not just a convenience: cards with more values offer more flexibility and are inherently more valuable. The auto-selection logic picks the qualifying value closest to the range center, falling back to the first value if none qualify.

### Win and Loss Conditions

- **Win**: `turns_won ≥ wins_required` within `max_turns` → base Fish token reward × FishAmount
- **Loss**: Max turns exhausted without enough wins, FishingDurability ≤ 0, or all hand cards unpayable

### Encounter-Scoped Tokens

| Token | Description |
|-------|-------------|
| FishingRangeMin | Lower bound of valid range |
| FishingRangeMax | Upper bound of valid range |
| FishAmount | Reward multiplier — scales the base reward on encounter win (does NOT change the number of counted wins) |

### FishAmount as Reward Multiplier

FishAmount starts at a baseline value and can be modified by card effects. When the encounter ends with a win, the base reward is multiplied by the current FishAmount value. This means FishAmount is a **reward optimisation lever**, not a win-counting shortcut. Cards that boost FishAmount trade immediate value-play for higher payoff on the final reward.

## Fish Deck Composition

The fish deck contains cards with a weighted distribution across several value tiers. Lower-value fish are more common, creating a base win rate, while higher-value fish test the player's range management and card selection. The exact values and distribution are configuration-driven — see `configurations/fishing/cards.json`.

## Token Lifecycle in Fishing

- **FishingDurability**: Persistent counter. Decreased by post-play costs. Triggers encounter loss if ≤ 0. Persists across encounters — total durability is the session budget. **Note**: The initial durability value is a testing shortcut; after rest encounter balancing, the starting value will likely be significantly lower (closer to one-tenth of the current value).
- **Stamina**: Persistent counter. Pre-play cost on advanced cards. Persists across encounters; main recovery comes from resting.
- **Health**: Persistent counter. Pre-play cost on high-tier cards. Persists across encounters.
- **FishingRangeMin/FishingRangeMax**: Encounter-scoped. Modified by range-modifier effects on player cards. Resets each encounter.
- **FishAmount**: Encounter-scoped. Modified by amount-modifier effects. Scales the base reward on win. Resets each encounter.

## Player Card Tiers

Cards span several tiers with increasing value ranges, higher costs, and stronger modifiers. Basic cards have low durability cost and simple values; mid-tier cards add stamina costs and may include range adjustments; high-tier cards add health costs and may include range narrowing or FishAmount boosts. The exact values are configuration-driven.

## Simulation Results

### Tuned Results (B2.7 — Fish reward = 15)

Simulation: 3 games × 20 encounters per game, seed 42. DisciplineDriver pattern with
effect-type-aware strategies (effect_id → template lookup for card classification).

| Strategy | Win Rate | Total Yield | Total Durability | Yield/Durability | Target |
|----------|----------|-------------|-----------------|-----------------|--------|
| Random | 98.3% | 7,126 | 20,662 | 0.345 | 0.2–0.4 ✅ |
| Greedy | 83.3% | 5,588 | 22,002 | 0.254 | 0.2–0.4 ✅ |
| Conservative | 81.7% | 5,488 | 18,301 | 0.300 | 0.2–0.4 ✅ |
| Tactician | 83.3% | 5,588 | 22,002 | 0.254 | 0.2–0.4 ✅ |

### Key Findings (Tuned)

1. **All strategies within target** — yield/durability ranges from 0.254 to 0.345, well within the 0.2–0.4 band.
2. **Primary tuning lever**: Reducing the base Fish reward from 1,000 to 15 (≈67× reduction) brought yield into range. Option C from the original tuning recommendations proved simplest and most effective.
3. **Win rates are high** (82–98%) — the fish deck composition (66% of cards in valid range) makes encounter wins common. Strategy differentiation comes from value selection, not win rate.
4. **Random outperforms targeted strategies** — random card selection accidentally produces good range coverage, while Greedy/Tactician's in-range preference concentrates on specific values. This is an acceptable inversion for simple strategies: the "skill" in fishing is reading the range, not just picking high values.
5. **Conservative is efficient** — lowest durability spend (18,301 vs 22,002) yields a competitive ratio. Cheap cards preserve the durability budget.
6. **Greedy = Tactician** — with current configs, the Tactician's range/FishAmount levers don't yet differentiate it from Greedy. Future configs with more range-modifier and FishAmount cards could widen this gap.

### Tuning History

| Round | Reward | Random | Greedy | Conservative | Tactician | Result |
|-------|--------|--------|--------|-------------|-----------|--------|
| Baseline | 1000 | 23.1 | 17.0 | 19.8 | 17.0 | All way too high |
| Sweep 1 | 12 | 0.276 | 0.207 | 0.250 | 0.207 | ✅ All pass (low end) |
| Sweep 1 | 18 | 0.414 | 0.306 | 0.359 | 0.306 | ❌ Random over |
| Sweep 1 | 25 | 0.573 | 0.424 | 0.491 | 0.424 | ❌ All over |
| Sweep 2 | 14 | 0.319 | 0.238 | 0.276 | 0.238 | ✅ All pass |
| **Sweep 2** | **15** | **0.345** | **0.254** | **0.300** | **0.254** | **✅ Selected** |
| Sweep 2 | 16 | 0.258 | 0.258 | 0.310 | 0.258 | ✅ All pass |

### Future Tuning Directions

- **Strategy differentiation**: Add more range-modifier and FishAmount-modifier cards to reward Tactician's multi-lever optimization. Currently Tactician = Greedy in yield.
- **Fish deck tuning**: Shift fish deck toward higher values (fewer in-range cards) to lower win rates and increase the skill gap between Random and targeted strategies.
- **Durability budget**: Once rest encounters are balanced, reduce initial FishingDurability (~1/10 current) to make durability preservation more strategic.

## Config Parameters

Key fishing config parameters in `configurations/fishing/cards.json`:
- Player card value ranges (min/max per tier) and number of values per card
- Fish deck value distribution (count per value level)
- Wins required per encounter
- Max turns per encounter
- Initial valid range (FishingRangeMin, FishingRangeMax)
- Range modifier limits (min/max shifts)
- FishAmount modifier limits
- Durability cost ranges (min/max per tier)
- Stamina cost ranges (min/max per tier)
- Health cost ranges (min/max per tier)
- Base Fish token reward per win

## Tuning Tips

- **Valid range width is the primary balance lever**: A wider range makes it easier to land wins; a narrower range increases difficulty. The interaction between initial range width and card-based range modifiers determines how much tactical play matters.
- **Fish deck distribution determines base difficulty**: More low-value fish makes the encounter easier because more player cards will land in range. More high-value fish forces players to use expensive high-tier cards or rely on best-matching multi-value selection.
- **Three tactical levers (not just one)**: Players optimise through (1) **best-matching value selection** — multi-value cards give flexibility to match different fish, (2) **range management** — expanding the valid range makes subsequent turns easier, and (3) **FishAmount boosting** — trading turns for higher reward scaling. All three levers should contribute meaningfully to yield-per-durability.
- **FishAmount as reward multiplier**: Cards that boost FishAmount increase the final reward. Balance FishAmount modifiers carefully — too accessible and they trivialize the encounter; too rare and the lever becomes irrelevant.
- **Dual loss conditions**: Both turn exhaustion and durability depletion can end an encounter as a loss. The balance between max_turns and durability cost per turn determines which is the binding constraint.
- **Durability budget**: Total durability across all fishing encounters bounds the session. The per-card durability cost relative to other disciplines determines how many encounters fishing supports.
- **Tiered balance enforcement**: Tactical play (combining range management, best-matching value selection, and FishAmount optimization) must produce more yield per durability than random value selection. If strategies converge, increase the impact of range modifiers or add cards with stronger multi-lever trade-offs.
- **RNG Coupling**: Changing card counts changes the RNG state for the entire game. Only aggregate metrics across many encounters are meaningful for comparison.
