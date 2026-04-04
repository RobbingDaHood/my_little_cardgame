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
| Full Utilization | 2 (Tactical) | Uses all card types optimally — power cards for yield, light cards for light management, stamina cards for sustain. No card is worthless except insight cards (which increase difficulty by design). Requires light-management mechanics that reward multi-round play. |

- Simple strategies (Random, Greedy, Conservative) should all produce somewhat similar yield-per-durability ratios within the tier 1 range (0.5–2.0).
- Tactician strategies should achieve measurably higher yield per durability than any simple strategy, landing in the tier 2 range (1.5–4.0).
- The gap between the best simple strategy and the tactician should be meaningful — tactical light-level management and conclude-timing must be rewarded.
- The **Full Utilization** strategy is an aspirational target: the game mechanics should be tuned so that every card type has a situation where playing it is optimal. If a strategy that ignores a card type always outperforms one that uses it, the mechanic needs redesign. The only exception is insight cards, which intentionally increase difficulty.

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

## Light Level as Ramping Yields

MiningLightLevel is intended to be the **ramping yields mechanic** — playing light cards should enable multi-round strategies that accumulate more yield than single-round conclude strategies. For this to work, the balance must ensure:

- **Light cards must pay back their round cost**: Playing a light card costs one round of ore damage (durability) for 0 immediate yield. The light boost from that card must generate enough extra yield in subsequent rounds to exceed the durability cost of the wasted round. If this condition isn't met, light cards become worthless and all strategies converge on "play one power card and conclude."
- **Multi-round play must be viable**: The expected yield-per-durability of a multi-round strategy (power → light → power → conclude) should be competitive with the single-round strategy (power → conclude at peak light). This may require: lower initial light (so boosting it matters), slower light decay per round, or light cards that also grant partial yield.
- **Insight cards are the exception**: Insight cards are designed to increase encounter difficulty. They are the only card type that should be strategically undesirable to play.

### Design Directions for Making Light Management Strategic

If light cards are currently non-viable, consider these config adjustments:
1. **Lower initial light level** — if light starts low, playing light cards becomes necessary to reach efficient mining levels, rather than starting at peak and only declining.
2. **Light cards grant partial yield** — add a small MiningPower component to light cards so they produce some yield while boosting light.
3. **Slower light decay** — reduce ore card light-reduction effects so multi-round encounters are cheaper, making the light investment worthwhile.
4. **Light threshold bonus** — add a mechanic where yield scales nonlinearly with light (e.g., bonus above a threshold), rewarding players who invest in light management.

**Note**: After implementing any of these mechanic changes, the balance must be re-tuned. Update this document with the new rules before running the balancing step.

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
- **Compound ore effects**: Ore cards can have MULTIPLE effects. For example, "ore_light_medium" cards have both light reduction AND durability damage. Don't assume 1 card = 1 effect when calculating expected costs per round. At the start of tuning, enumerate the ore deck and compute expected durability cost per round (weighted by card frequency), expected light loss per round, and the fraction of cards with compound effects.
- **Lumber-to-durability conversion**: Some mining cards cost Lumber. The effective durability formula is `effective_durability = raw_durability + (lumber_consumed / WOODCUTTING_YIELD_PER_DURABILITY)`. Lumber-cost cards can massively inflate effective durability. A strategy that spends large amounts of lumber may look efficient on raw yield but terrible on effective yield/dur.

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
