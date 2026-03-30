# Mining Balance

This document contains mining-specific balancing information. It is the authoritative reference for mining balance targets, mechanics, and tuning guidance.

## Target Metrics

Mining balance is measured by **yield per durability** — how much Ore a player extracts for the MiningDurability spent across encounters. Unlike combat (which uses win streaks), gathering disciplines focus on resource efficiency over a session of encounters.

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
| Random | 1 (Simple) | Plays any available mining card without considering light level or costs |
| Greedy | 1 (Simple) | Always plays the highest-power card available |
| Conservative | 1 (Simple) | Plays lowest-cost cards to preserve durability |
| Tactician | 2 (Tactical) | Plays only power cards while light is high, aborts encounters without power in hand, concludes at light < 140 |
| Durability Tactician | 2 (Tactical) | Plays only free power cards, never plays cost or light cards, concludes immediately when no free power available |

- Simple strategies (Random, Greedy, Conservative) should all produce somewhat similar yield-per-durability ratios within the tier 1 range (0.5–2.0).
- Tactician strategies should achieve measurably higher yield per durability than any simple strategy, landing in the tier 2 range (1.5–4.0).
- The gap between the best simple strategy and the tactician should be meaningful — tactical light-level management and conclude-timing must be rewarded.

### Tier-2 Strategy Requirements

There must be at least **2 distinct tier-2 runners** that outperform all tier-1 strategies:

1. **Yield-optimizer**: A tactician that exploits yield-boosting card effects (e.g., plays yield-enhancing cards when light level is high, times conclude for maximum ore).
2. **Non-yield tactician**: A tactician that beats tier-1 runners **without ever adjusting yield outcome** — it must never rely on yield-boosting effects. Instead, it wins through superior resource management (durability conservation, stamina efficiency, optimal encounter conclusion timing).

Choosing the correct encounter via scouting is a valid additional tier-2 tactic, but it does **not** count toward the "non-yield tactician" requirement. There must always be at least one tier-2 runner that beats tier-1 purely through in-encounter play decisions.

### Cross-Discipline Yield Parity

Mining should produce roughly the same yield value per durability as other gathering disciplines (herbalism, woodcutting, fishing), even though the yield type (Ore) differs. A discipline may use more encounters than another, as long as the average yield per durability is comparable.

## Mining Mechanics

### Core Formula

The mining yield formula is multiplicative:
```
yield += mining_power × light_level / 100
```

- **MiningPower** is a transient trigger — it is gained from playing mining cards, immediately converted to yield via the formula, and not stored.
- **MiningLightLevel** is the primary multiplier — it starts at a configured value and changes based on ore card effects and player cards.
- High light level amplifies all power gains; low light level makes mining inefficient.

### Reward Calculation

At encounter conclusion (or durability depletion):
```
ore_reward = min(Stamina, MiningYield)
```

Stamina is consumed equal to the ore gained. This creates a secondary cap: even if yield is high, low stamina limits the reward. Players must manage stamina across encounters.

### Encounter Flow

1. Player plays a mining card → pre-play costs deducted (Stamina, Lumber)
2. Card effects processed: MiningPower gain → yield formula applied; MiningLightLevel gain → light adjusted
3. Ore card plays automatically (random from ore deck), player auto-draws 1 mining card
4. Check end conditions: durability ≤ 0 (rewards granted, encounter ends as loss), health ≤ 0, or all hand cards unpayable
5. Player may voluntarily conclude → reward calculated and granted

### Durability Depletion

If MiningDurability reaches 0, the encounter ends immediately as a loss, but **rewards are still granted** — the same ore_reward = min(Stamina, MiningYield) calculation applies and stamina cost is deducted. This means running out of durability is costly on the record but the player keeps what they mined.

### Voluntary Conclusion

The player controls when to conclude the encounter. This is a key tactical lever: conclude too early and you leave yield on the table; continue too long and you risk durability depletion.

## Token Lifecycle in Mining

- **MiningLightLevel**: Encounter-scoped. Starts at an initial configured value. Modified by both player and ore card effects. Higher light = more yield per power. Resets each encounter.
- **MiningYield**: Encounter-scoped. Accumulates from 0 during the encounter. Converted to Ore reward on conclusion or durability depletion. Resets each encounter.
- **MiningDurability**: Persistent counter. Decreases from ore card post-play effects. Triggers encounter end (with rewards) if ≤ 0. Persists across encounters — total durability is the session budget. **Note**: The initial durability value is a testing shortcut; after rest encounter balancing, the starting value will likely be significantly lower (closer to one-tenth of the current value).
- **Stamina**: Persistent counter. Pre-play cost on player cards. Also consumed at conclusion to cap Ore reward. Persists across encounters; main recovery comes from resting.
- **Lumber**: Persistent counter. Pre-play cost on some mining cards. Represents tool wear.
- **Health**: Persistent counter. Rare damage from heavy ore effects. Persists across encounters.

## Ore Deck Composition

The ore deck contains cards across several tiers spanning light-reduction, durability-damage, and health-damage effects. The distribution is weighted toward lower-impact cards that pace the encounter, with occasional high-impact threats. The exact composition is configuration-driven — see `configurations/mining/cards.json`.

## Simulation Results

Current simulation results from `make balance-check` (3 games × 20 encounters per strategy, seed 42):

| Strategy | Tier | Yield/Dur | Target | Rounds/Enc | Status |
|----------|------|-----------|--------|------------|--------|
| Random | 1 | 1.10 | 0.5–2.0 | 2.6 | ✅ |
| Greedy | 1 | 0.51 | 0.5–2.0 | 8.9 | ✅ |
| Conservative | 1 | 1.38 | 0.5–2.0 | 0.4 | ✅ |
| Tactician | 2 | 2.27 | 1.5–4.0 | 0.5 | ✅ |
| Dur. Tactician | 2 | 2.17 | 1.5–4.0 | 0.5 | ✅ |

### Strategy Observations

- **Random** achieves solid efficiency (1.10) because the 53% power-card composition means most random plays generate yield, and encounters auto-end at light=0 after ~2.6 rounds.
- **Greedy** is the least efficient (0.51) — it plays every card including non-power cards for many rounds, wasting durability on 0-yield rounds.
- **Conservative** is highly efficient (1.38) by avoiding cost cards, which limits it to ~0.4 rounds per encounter but ensures every play is efficient.
- **Tactician** achieves the best yield/dur (2.27) by playing ONLY power cards and aborting encounters with no power in hand. The key insight: playing non-power cards wastes a round (~19 durability) for 0 yield, so the optimal strategy never plays light or utility cards.
- **Durability Tactician** matches the Tactician (2.17) through the same core mechanic — play one free power card at peak light and conclude immediately. It differs by never playing cost power cards even at high light.

### Key Balance Dynamics

- **Light cards are a trap**: Each round an ore card plays (~19 avg durability cost) regardless of what the player plays. A light card gives 0 yield that round. Even though it boosts light for future rounds, the extra durability cost exceeds the yield benefit.
- **Ore deck has compound effects**: 20 of 50 ore cards have BOTH light reduction AND durability damage (medium tier), making each round more costly than the light-only cards suggest.
- **Tier separation comes from conclude timing**: T2 strategies abort unprofitable encounters immediately, while T1 strategies play cards indiscriminately or stay too long.

## Config Parameters

Key mining config parameters in `configurations/mining/cards.json`:
- Initial light level (via encounter state)
- Player card power values (min/max on mining effects)
- Player card light-level modifiers
- Pre-play costs: Stamina (min/max), Lumber (min/max)
- Ore card durability damage (min/max per tier)
- Ore card light reduction (min/max per tier)
- Ore card health damage (min/max, rare tier only)

## Tuning Tips

- **Light level is the primary balance lever**: Since yield = power × light / 100, small changes to initial light level or light-reduction rates dramatically affect total yield. Light level is multiplicative — a percentage change in light changes all subsequent yield by the same percentage.
- **Stamina cap interaction**: Even with high yield, stamina limits the actual Ore gained. Balance stamina costs against stamina recovery from resting to control session throughput.
- **Durability as session budget**: Total durability across all mining encounters bounds the session. Higher per-encounter yield with fewer encounters should roughly equal lower per-encounter yield with more encounters.
- **Durability depletion grants rewards**: Unlike a voluntary abort, running out of durability still triggers the reward calculation. This makes durability management an efficiency concern (ending encounters with leftover yield potential) rather than a catastrophic-loss concern.
- **Ore deck composition**: More heavy/health ore cards increases loss risk but doesn't change yield-per-successful-encounter. Adjust ore deck to control loss rate, not yield rate.
- **Tiered balance enforcement**: Tactical light-level management (boosting light before power plays, timing conclusion) must produce measurably more yield per durability than random/greedy play. If strategies converge, add mechanics that reward timing (e.g., light-level thresholds, combo bonuses).
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
- Any CardEffect within mining (including suggesting new CardEffects as a last resort)
- Any Card within mining (including suggesting new Cards, but try without first)
- Any encounter within mining (including suggesting new encounters, but try without first)

Balance mutators **must NOT** change:
- Starting Health, Stamina, or any player starting tokens
- Health or Stamina after death
- Hand sizes (all must remain 5)
- Deck sizes (all must remain 50)
- Anything outside the mining discipline

### Deck and Hand Sizing

- All player deck hand sizes: **5** (controlled by per-deck MaxHand tokens)
- All player deck sizes: **50** (controlled by per-deck MaxDeck tokens)
- Do NOT change deck or hand sizes to fix balance issues — adjust card effects and encounter parameters instead.
