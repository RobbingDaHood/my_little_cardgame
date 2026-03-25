# Mining Balance

This document contains mining-specific balancing information. It is the authoritative reference for mining balance targets, mechanics, and tuning guidance.

## Target Metrics

Mining balance is measured by **yield per durability** — how much Ore a player extracts for the MiningDurability spent across encounters. Unlike combat (which uses win streaks), gathering disciplines focus on resource efficiency over a session of encounters.

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
- **MiningLightLevel** is the primary multiplier — it starts at a configured value (default 300) and changes based on ore card effects and player cards.
- High light level amplifies all power gains; low light level makes mining inefficient.

### Reward Calculation

At encounter conclusion:
```
ore_reward = min(Stamina, MiningYield)
```

Stamina is consumed equal to the ore gained. This creates a secondary cap: even if yield is high, low stamina limits the reward. Players must manage stamina across encounters.

### Encounter Flow

1. Player plays a mining card → pre-play costs deducted (Stamina, Lumber)
2. Card effects processed: MiningPower gain → yield formula applied; MiningLightLevel gain → light adjusted
3. Ore card plays automatically (random from ore deck), both sides draw
4. Check end conditions: durability ≤ 0, health ≤ 0, or all hand cards unpayable
5. Player may voluntarily conclude → reward calculated and granted

### Voluntary Conclusion

The player controls when to conclude the encounter. This is a key tactical lever: conclude too early and you leave yield on the table; continue too long and you risk durability depletion (loss = no reward).

## Token Lifecycle in Mining

- **MiningLightLevel**: Encounter-scoped. Starts at initial_light_level (default 300). Modified by both player and ore card effects. Higher light = more yield per power. Resets each encounter.
- **MiningYield**: Encounter-scoped. Accumulates from 0 during the encounter. Converted to Ore reward on conclusion. Resets each encounter.
- **MiningDurability**: `PersistentCounter` (initialized at 10,000). Decreases from ore card post-play effects (80–250 per ore card). Triggers encounter loss if ≤ 0. Persists across encounters — total durability is the session budget.
- **Stamina**: `PersistentCounter`. Pre-play cost (80–200 per player card). Also consumed at conclusion to cap Ore reward. Persists across encounters; main recovery comes from resting.
- **Lumber**: `PersistentCounter`. Pre-play cost on some mining cards (10–30). Represents tool wear.
- **Health**: `PersistentCounter`. Rare damage from heavy ore effects (50–100). Persists across encounters.

## Ore Deck Composition

50 ore cards across 5 tiers:

| Tier | Count | Primary Effect | Balance Role |
|------|-------|---------------|-------------|
| Light-small | 15 | Small light reduction | Common, low impact |
| Light-medium | 20 | Medium light reduction | Most frequent, pacing control |
| Durability-medium | 10 | Medium durability damage | Durability pressure |
| Heavy | 5 | Large durability + light reduction | High threat, rare |
| Health | 5 | Direct health damage | Rare but dangerous |

The ore deck distribution controls encounter pacing: mostly light-impact cards with occasional durability/health threats.

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

- **Light level is the primary balance lever**: Since yield = power × light / 100, small changes to initial light level or light-reduction rates dramatically affect total yield. Light level is multiplicative — a 10% change in light changes all subsequent yield by 10%.
- **Stamina cap interaction**: Even with high yield, stamina limits the actual Ore gained. Balance stamina costs against stamina recovery from resting to control session throughput.
- **Durability as session budget**: 10,000 durability across all mining encounters means the number of encounters (and thus total yield) is bounded. Higher per-encounter yield with fewer encounters should roughly equal lower per-encounter yield with more encounters.
- **Ore deck composition**: More heavy/health ore cards increases loss risk but doesn't change yield-per-successful-encounter. Adjust ore deck to control loss rate, not yield rate.
- **Tiered balance enforcement**: Tactical light-level management (boosting light before power plays, timing conclusion) must produce measurably more yield per durability than random/greedy play. If strategies converge, add mechanics that reward timing (e.g., light-level thresholds, combo bonuses).
- **RNG Coupling**: Changing card counts changes the RNG state for the entire game. Only aggregate metrics across many encounters are meaningful for comparison.
