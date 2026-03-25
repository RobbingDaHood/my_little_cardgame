# Combat Balance

This document contains combat-specific balancing information. It is the authoritative reference for combat balance targets, mechanics, and tuning guidance.

## Target Metrics

Combat balance is measured by **consecutive win streaks** (how many combats a player wins before dying), not per-combat win rate. Streak length reflects sustained resource management across encounters.

### Strategy Hierarchy (streak)

| Strategy | Min Streak | Max Streak | Description |
|----------|-----------|-----------|-------------|
| Random | 3.0 | 8.0 | Uniform random card selection |
| Greedy | 3.0 | 7.5 | Picks highest rolled value |
| Conservative | 2.5 | 7.0 | Picks lowest non-cost card |
| Tactician-greedy | 3.0 | 18.0 | Enemy-aware, picks most damaging card |
| Tactician-conservative | 3.0 | 18.0 | Enemy-aware, picks most defensive card needed |

- The gap between the best simple strategy and any tactician variant should be ≥1 streak length. **Current finding (B2.1):** tactician does NOT outperform simple strategies — the game mechanics don't yet reward tactical play enough. This is a known gap; future work should add mechanics that differentiate skilled play (e.g., timing-dependent shields, combo effects).
- Random, Greedy, and Conservative should all have somewhat similar performance ranges.
- The maximum of the simple tier may overlap with the minimum of the Tactician tier, but no more than that.
- All Tactician variants should perform somewhat equally but always better than non-tacticians.

### Win Rates

Combat balance is measured by **consecutive win streaks**, not per-combat win rates. Win rates can appear structurally high due to easy initial encounters and scouting mechanics, but they do not reflect per-encounter difficulty. Streak length is the authoritative metric for sustained resource management across encounters.

## Combat Mechanics

### Damage Resolution Order

1. **Dodge** absorbs first (timing-based, expires after Defending phase)
2. **Shield** absorbs next (expires at combat encounter end — does NOT persist across encounters)
3. **Health** takes remaining damage

### Token Lifecycle in Combat

- **Dodge**: `FixedTypeDuration { duration: 1, phases: [Defending] }` — high absorption per card, but temporary. Well-timed dodge blocks a full attack; wasted if enemy doesn't attack. Rewards precise timing.
- **Shield**: Expires at end of combat encounter. Within a single combat, shield persists through all rounds. Between encounters, shield is cleared. This means shield provides steady damage reduction during a fight but does not create compounding advantages across encounters.
- **Health**: `PersistentCounter` — persists across encounters. Only decreases during combat (except on death reset or via healing resource cards).
- **Stamina**: `PersistentCounter` — persists across encounters. Moderate recovery possible via resource cards, but main recovery comes from resting.

### Card Persistence

Cards are never reset between encounters — deck, hand, and discard states carry over. Card depletion over a full game session is a critical balancing dimension. When a card is drawn from the deck and there are no more cards to draw, the full discard pile is moved into the deck. Because cards are randomized when drawing from deck to hand, there is no additional randomization needed when moving discard to deck.

In combat, resource cards that draw new cards are the primary mechanism to avoid hand depletion. Adjust the card gain from relevant resource cards to avoid card depletion — do NOT change deck or hand sizes for this purpose.

### Healing and Stamina Recovery

Moderate healing and stamina recovery cards exist in the resource deck, allowing small amounts of recovery during combat. However, the main healing and stamina gain should come from resting encounters. Combat resource cards provide supplemental recovery only.

## Card Cost Distribution Philosophy

All combat decks follow a three-tier cost distribution that creates a risk/reward spectrum:

### Abundance Tiers

1. **Free cards** (no cost) — the most abundant in every deck (50-60%). These are the bread-and-butter plays with moderate effect values.
2. **Stamina-cost cards** — the second most common tier (20-30%). These cost Stamina but always outperform free cards in raw effect value.
3. **Health-cost cards** — rare but devastating (5-10%). These sacrifice Health but deal enough damage to justify the cost most times.

### Card Tier Value Hierarchy

For each deck type, effect values follow a strict ordering:

```
free_max < stamina_cost_min < health_cost_min
```

This ensures that cost cards are always worth playing when affordable. The "crit" free cards (natural high rolls from the same CardEffect template) can never exceed cost card minimums.

Within the free tier, "moderate" and "crit" cards share the same CardEffect template — the variance comes from independent per-copy rolls across the template's min-max range. Having many "main" copies and fewer "crit" copies documents design intent without creating separate mechanics.

### Current Combat Card Distribution

**Attack Deck (50 cards):**

| Card Type | Effect | Range | Cost | Count | % |
|-----------|--------|-------|------|-------|---|
| Main free | deal_damage | 200–400 | None | 28 | 56% |
| Crit free | deal_damage | 200–400 | None | 6 | 12% |
| Stamina-cost | stamina_damage | 420–550 | Stamina 15-20% | 11 | 22% |
| Health-cost | health_damage | 500–750 | Health 15-25% | 5 | 10% |

**Defence Deck (50 cards, 50/50 shield/dodge):**

| Card Type | Effect | Range | Cost | Count | % |
|-----------|--------|-------|------|-------|---|
| Shield moderate | grant_shield | 80–230 | None | 13 | 26% |
| Shield crit | grant_shield | 80–230 | None | 5 | 10% |
| Shield stamina | stamina_shield | 240–320 | Stamina 15-20% | 7 | 14% |
| Dodge moderate | grant_dodge | 350–650 | None | 11 | 22% |
| Dodge crit | grant_dodge | 350–650 | None | 4 | 8% |
| Dodge stamina | stamina_dodge | 670–900 | Stamina 15-20% | 10 | 20% |

**Resource Deck (50 cards, split focus):**

| Card Type | Effects | Count | % |
|-----------|---------|-------|---|
| Draw-only | draw_cards (3/3/3) | 22 | 44% |
| Stamina-only | minor_stamina (50-150) | 10 | 20% |
| Heal-only | heal_health (50-150) | 5 | 10% |
| Stamina + Draw | minor_stamina + draw_cards | 7 | 14% |
| Heal + Draw | heal_health + draw_cards | 3 | 6% |
| Insight | insight (1-5) | 3 | 6% |

**Enemy Decks (50 cards each):**

| Deck | Effect | Range | Deck/Hand |
|------|--------|-------|-----------|
| Attack | enemy_damage | 300–420 | 40/10 |
| Defence | enemy_shield | 100–200 | 40/10 |
| Resource | enemy_stamina (80-120) + enemy_draw (1/1/1) | — | 40/10 |

## Config Parameters

Key combat config parameters in `configurations/combat/cards.json`:
- Enemy HP (via `initial_tokens`)
- Dodge values (min/max on defence cards)
- Shield values (min/max on defence cards)
- Damage values (min/max on attack cards)
- Cost-damage values (attack cards with stamina or HP cost)
- Draw card counts (resource cards)
- Healing/stamina amounts (resource cards, minor only — primary recovery from rest)

## Strategy Tier Definitions

- **Simple tier** (random, greedy, conservative): No encounter-state awareness. Picks cards based on value or cost avoidance only. Random selects uniformly; greedy picks highest value; conservative picks lowest non-cost. Target streak range: 3.0–8.5.
- **Intermediate tier** (tactician): Reads combat phase and card types. Picks dodge for defence (avoids cost), cost_damage for attack (high burst), highest value for resource. Target streak range: 7.5–18.0.
- **Advanced tier** (future: meta-strategist): Manages resources across encounters. Plans card usage over multiple combats, considers scouting difficulty scaling and HP attrition.

## Baseline Findings (B2.1)

Initial 1000-game simulation (seed 42, 3 strategies × 20 max encounters) showed ~99% combat win rate across all strategies with ~3 encounters per game before stamina depletion. This confirmed combat was significantly too easy relative to targets. Key observations: all strategies performed nearly identically, games terminated due to stamina depletion not death, and combat rebalancing needed to focus on enemy damage scaling, stamina economy, and card differentiation. Subsequent 35-iteration tuning addressed these issues.

## Tuning Tips

- **RNG Coupling**: Changing card counts changes the RNG state for the entire game. Results from different configs are not directly comparable per-encounter. Only aggregate metrics are meaningful.
- **Shield within combat**: Since shield expires at combat end, shield values can be somewhat higher per card without creating compounding advantages across encounters. The balance lever is the relationship between enemy damage per round and shield grant per round within a single combat.
- **Death spiral interaction**: After player death, scouting generates easier encounters (configurable reduction). This prevents compounding difficulty from making the game unwinnable.
- **Scouting mutation asymmetry**: Enemy HP scales fully with difficulty factor, but enemy card effects only scale probabilistically (~10% per step). This creates "HP sponge" encounters at high difficulty. Future work should address proportional scaling (see scouting balance).
