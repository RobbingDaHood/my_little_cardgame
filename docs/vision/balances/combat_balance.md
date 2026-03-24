# Combat Balance

This document contains combat-specific balancing information. It is the authoritative reference for combat balance targets, mechanics, and tuning guidance.

## Target Metrics

Combat balance is measured by **consecutive win streaks** (how many combats a player wins before dying), not per-combat win rate. Streak length reflects sustained resource management across encounters.

### Strategy Hierarchy (streak)

| Strategy | Min Streak | Max Streak | Description |
|----------|-----------|-----------|-------------|
| Random | 3.5 | 8.0 | Uniform random card selection |
| Greedy | 3.0 | 7.0 | Picks highest rolled value |
| Conservative | 2.5 | 6.0 | Picks lowest non-cost card |
| Tactician-greedy | 8.0 | 18.0 | Enemy-aware, picks most damaging card |
| Tactician-conservative | 8.0 | 18.0 | Enemy-aware, picks most defensive card needed |

- The gap between the best simple strategy and any tactician variant should be ≥1 streak length.
- Random, Greedy, and Conservative should all have somewhat similar performance ranges.
- The maximum of the simple tier may overlap with the minimum of the Tactician tier, but no more than that.
- All Tactician variants should perform somewhat equally but always better than non-tacticians.

### Win Rates

Win rates are structurally high (55-95%) because: (1) initial encounters are easy wins, (2) scouting mutation only scales ~10% of enemy card effects per step while HP fully scales, (3) the death spiral is self-limiting. Win rates primarily measure death spiral severity, not per-encounter difficulty.

- Random: 55-80%
- Greedy: 45-65%
- Conservative: 70-95% (shield + basic attack is safe but slow)

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

## Config Parameters

Key combat config parameters in `configurations/combat/cards.json`:
- Enemy HP (via `initial_tokens`)
- Dodge values (min/max on defence cards)
- Shield values (min/max on defence cards)
- Damage values (min/max on attack cards)
- Cost-damage values (attack cards with HP cost)
- Draw card counts (resource cards)
- Healing/stamina amounts (resource cards)

## Tuning Tips

- **RNG Coupling**: Changing card counts changes the RNG state for the entire game. Results from different configs are not directly comparable per-encounter. Only aggregate metrics are meaningful.
- **Shield within combat**: Since shield expires at combat end, shield values can be somewhat higher per card without creating compounding advantages across encounters. The balance lever is the relationship between enemy damage per round and shield grant per round within a single combat.
- **Death spiral interaction**: After player death, scouting generates easier encounters (configurable reduction). This prevents compounding difficulty from making the game unwinnable.
- **Scouting mutation asymmetry**: Enemy HP scales fully with difficulty factor, but enemy card effects only scale probabilistically (~10% per step). This creates "HP sponge" encounters at high difficulty. Future work should address proportional scaling (see scouting balance).
