# Fishing Balance

This document contains fishing-specific balancing information. It is the authoritative reference for fishing balance targets, mechanics, and tuning guidance. **Simulation results belong in the PR description of the balancing PR, not in this document.** Only goals, mechanics, config parameters, and tuning tips should go here.

## Target Metrics

Fishing balance is measured by **yield per durability** — how many Fish tokens a player earns for the FishingDurability spent across encounters. Unlike combat (which uses win streaks), gathering disciplines focus on resource efficiency over a session of encounters.

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
| Random | 1 (Simple) | Plays any available fishing card without considering fish value or range |
| Greedy | 1 (Simple) | Always plays the highest-value fishing card available |
| Conservative | 1 (Simple) | Plays lowest-cost cards to preserve durability |
| Tactician | 2 (Tactical) | Manages valid range (widening/narrowing), selects values that best match the current fish, and boosts FishAmount for reward scaling |

- Simple strategies (Random, Greedy, Conservative) should all produce somewhat similar yield-per-durability ratios within the tier 1 range (0.5–2.0).
- Tactician strategies should achieve measurably higher yield per durability, landing in the tier 2 range (1.5–4.0) — range management, best-matching value selection, and FishAmount optimization must all be rewarded.
- The gap must reflect the skill of reading the fish distribution and combining multiple optimisation levers.

### Tier-2 Strategy Requirements

There must be at least **2 distinct tier-2 runners** that outperform all tier-1 strategies:

1. **Yield-optimizer**: A tactician that exploits FishAmount boosting and range management to maximize Fish reward per encounter — trades immediate value-play for higher reward scaling.
2. **Non-yield tactician**: A tactician that beats tier-1 runners **without ever adjusting yield outcome** — it must never use FishAmount-boosting effects. Instead, it wins through superior value selection, win-rate optimization (landing more turns in range), and durability conservation.

Choosing the correct encounter via scouting is a valid additional tier-2 tactic, but it does **not** count toward the "non-yield tactician" requirement. There must always be at least one tier-2 runner that beats tier-1 purely through in-encounter play decisions.

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

## General Design Principles

These principles apply across all disciplines. For full details, see `docs/design/vision.md`.

### Card Cost Distribution

- **Free cards** (no cost) should be the most common card type in every deck.
- **Stamina-cost cards** should be moderately common and always outperform free cards in raw effect value.
- **Health-cost cards** should be rare but powerful, always outperforming stamina-cost cards.
- This creates a risk/reward spectrum: safe low-output plays → moderate-cost moderate-output → high-risk high-output.

### Mutator Scope

Balance mutators (the agents implementing balance changes) **may** change within their discipline:
- Any CardEffect within fishing (including suggesting new CardEffects as a last resort)
- Any Card within fishing (including suggesting new Cards, but try without first)
- Any encounter within fishing (including suggesting new encounters, but try without first)

Balance mutators **must NOT** change:
- Starting Health, Stamina, or any player starting tokens
- Health or Stamina after death
- Hand sizes (all must remain 5)
- Deck sizes (all must remain 50)
- Anything outside the fishing discipline

### Deck and Hand Sizing

- All player deck hand sizes: **5** (controlled by per-deck MaxHand tokens)
- All player deck sizes: **50** (controlled by per-deck MaxDeck tokens)
- Do NOT change deck or hand sizes to fix balance issues — adjust card effects and encounter parameters instead.
