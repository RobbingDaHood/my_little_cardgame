# Mining Balance

This document contains mining-specific balancing information. It is the authoritative reference for mining balance targets, mechanics, and tuning guidance.

## Target Metrics

Mining balance is measured by **yield per durability** — how much Ore a player extracts for the MiningDurability spent across encounters. Unlike combat (which uses win streaks), gathering disciplines focus on resource efficiency over a session of encounters.

### Yield-per-Durability Targets

All yield disciplines (mining, herbalism, woodcutting, fishing) share the same aggregate target: **2,000–4,000 yield tokens per 10,000 total durability spent** (0.2–0.4 yield per durability). The tactician strategy should reliably land in the upper half of this range, while simple strategies land in the lower half. These targets are tuned in the balance simulation step (see roadmap B2.4) and must be identical across disciplines to enable parallel balancing — if one discipline significantly over- or under-produces relative to this band, its config needs adjustment.

### Strategy Hierarchy (yield per durability)

| Strategy | Description |
|----------|-------------|
| Random | Plays any available mining card without considering light level or costs |
| Greedy | Always plays the highest-power card available |
| Conservative | Plays lowest-cost cards to preserve durability |
| Tactician | Manages light level, times power plays when light is high, concludes at optimal moments |

- Simple strategies (Random, Greedy, Conservative) should all produce somewhat similar yield-per-durability ratios.
- Tactician strategies should achieve measurably higher yield per durability than any simple strategy.
- The gap between the best simple strategy and the tactician should be meaningful — tactical light-level management and conclude-timing must be rewarded.

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
3. Ore card plays automatically (random from ore deck), both sides draw
4. Check end conditions: durability ≤ 0 (rewards granted, encounter ends as loss), health ≤ 0, or all hand cards unpayable
5. Player may voluntarily conclude → reward calculated and granted

### Durability Depletion

If MiningDurability reaches 0, the encounter ends immediately as a loss, but **rewards are still granted** — the same ore_reward = min(Stamina, MiningYield) calculation applies and stamina cost is deducted. This means running out of durability is costly on the record but the player keeps what they mined.

### Voluntary Conclusion

The player controls when to conclude the encounter. This is a key tactical lever: conclude too early and you leave yield on the table; continue too long and you risk durability depletion.

## Simulation Results (B2.4)

Results from the mining balance simulation (3 games × 20 encounters per strategy, seeds 42–44):

| Strategy | Yield/Durability | Win Rate | Avg Rounds/Enc | Total Yield | Total Durability |
|----------|-----------------|----------|---------------|-------------|------------------|
| Tactician | **0.339** | 36.7% | 24.6 | 99,099 | 292,376 |
| Greedy | 0.300 | 51.7% | 26.8 | 70,166 | 233,817 |
| Random | 0.280 | 70.0% | 3.2 | 14,186 | 50,574 |
| Conservative | 0.112 | 13.3% | 0.3 | 278 | 2,473 |

### Hierarchy

**Tactician (0.339) > Greedy (0.300) > Random (0.280) > Conservative (0.112)**

- Tactician's light management sustains high yield per round across long encounters.
- Greedy achieves decent yield but wastes durability on rounds where light has degraded.
- Random concludes quickly (3 rounds avg) — moderate efficiency by avoiding durability drain.
- Conservative barely plays (light drops below its conclude threshold after 1 encounter) — extremely low yield.

### Key Config Changes from Baseline

| Parameter | Baseline | Tuned | Rationale |
|-----------|----------|-------|-----------|
| mining_power (all) | 300–1200 | 7–22 | Bring yield/durability into 0.2–0.4 range |
| mining_light_gain (free) | N/A (new) | 100–160 | Added free light card so light management is possible without Lumber |
| mining_light_with_lumber | 200–400 | 150–250 | Moderate light gain for cost |
| ore_light_small | 20–40 | 30–60 | Increased light pressure per round |
| ore_light_medium | 40–60 | 50–90 | Increased light pressure per round |
| ore_durability_medium | 80–120 | 200–400 | Higher per-round durability cost |
| ore_durability_heavy | 150–250 | 400–700 | Higher per-round durability cost |
| ore_health | 50–100 | 10–30 | Reduced health damage (not a balance lever) |
| initial_light_level | 300 | 50 | Lower default; persistent MiningLightLevel (200) is the actual starting value |
| MiningLightLevel (token) | N/A (new) | 200 | Persistent starting light for all encounters |
| MiningDurability (token) | 10,000 | 100,000 | High budget so durability doesn't bottleneck during tuning |
| Stamina (token) | 1,000 | 50,000 | High budget so stamina doesn't cap during tuning |
| Lumber (token) | N/A (new) | 10,000 | Required for lumber-cost mining cards |

## Token Lifecycle in Mining

- **MiningLightLevel**: Persistent counter. Starts at a configured initial value (set in `tokens.json`). Modified by both player and ore card effects during encounters. The value carries across encounters — light management has long-term consequences. Higher light = more yield per power.
- **MiningYield**: Encounter-scoped. Accumulates from 0 during the encounter. Converted to Ore reward on conclusion or durability depletion. Resets each encounter.
- **MiningDurability**: Persistent counter. Decreases from ore card post-play effects. Triggers encounter end (with rewards) if ≤ 0. Persists across encounters — total durability is the session budget. **Note**: The initial durability value is a testing shortcut; after rest encounter balancing, the starting value will likely be significantly lower (closer to one-tenth of the current value).
- **Stamina**: Persistent counter. Pre-play cost on player cards. Also consumed at conclusion to cap Ore reward. Persists across encounters; main recovery comes from resting.
- **Lumber**: Persistent counter. Pre-play cost on some mining cards. Represents tool wear.
- **Health**: Persistent counter. Rare damage from heavy ore effects. Persists across encounters.

## Ore Deck Composition

The ore deck contains cards across several tiers spanning light-reduction, durability-damage, and health-damage effects. The distribution is weighted toward lower-impact cards that pace the encounter, with occasional high-impact threats. The exact composition is configuration-driven — see `configurations/mining/cards.json`.

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
