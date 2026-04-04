# Combat Balance

This document contains combat-specific balancing guidelines. It is the authoritative reference for combat balance targets, mechanics, and tuning principles. Specific numbers (effect values, cost percentages, exact card counts) live in the configuration files — this document describes the **design intent and constraints** that configs must satisfy.

## Target Metrics

Combat balance is measured by **consecutive win streaks** (how many combats a player wins before dying), not per-combat win rate. Streak length reflects sustained resource management across encounters.

### Strategy Hierarchy (streak)

| Strategy | Min Streak | Max Streak | Description |
|----------|-----------|-----------|-------------|
| Random | 3.0 | 8.0 | Uniform random card selection |
| Greedy | 3.0 | 7.5 | Picks highest rolled value |
| Conservative | 2.5 | 7.0 | Picks lowest non-cost card |
| Tactician-greedy | 8.0 | 18.0 | Enemy-aware, plays the most damaging card |
| Tactician-conservative | 8.0 | 18.0 | Enemy-aware, plays the most defensive card needed |

- Tactician variants must **always** outperform all simple-tier strategies. The minimum tactician streak should be at or above the maximum simple-tier streak.
- Random, Greedy, and Conservative should all have somewhat similar performance ranges.
- All Tactician variants should perform somewhat equally but always better than non-tacticians.

### Win Rates

Win rates are structurally high because: (1) initial encounters are easy wins, (2) scouting mutation only scales a fraction of enemy card effects per step while HP fully scales, (3) the death spiral is self-limiting. Win rates primarily measure death spiral severity, not per-encounter difficulty. Streak length is the authoritative metric for sustained resource management across encounters.

## Combat Mechanics

### Damage Resolution Order

1. **Dodge** absorbs first (timing-based, expires after Defending phase)
2. **Shield** absorbs next (expires at combat encounter end — does NOT persist across encounters)
3. **Health** takes remaining damage

### Token Lifecycle in Combat

- **Dodge**: High absorption per card, but temporary. Well-timed dodge blocks a full attack; wasted if enemy doesn't attack. Rewards precise timing.
- **Shield**: Expires at end of combat encounter. Within a single combat, shield persists through all rounds. Between encounters, shield is cleared. This means shield provides steady damage reduction during a fight but does not create compounding advantages across encounters.
- **Health**: Persists across encounters. Only decreases during combat (except on death reset or via healing resource cards).
- **Stamina**: Persists across encounters. Moderate recovery possible via resource cards, but main recovery comes from resting.

### Card Persistence (Combat-Specific)

Cards are never reset between encounters — deck, hand, and discard states carry over. **In Combat**, card depletion over a full game session is a critical balancing dimension (Combat is the only discipline without auto-draw). When a card is drawn from the deck and there are no more cards to draw, the full discard pile is moved into the deck. Because cards are randomized when drawing from deck to hand, there is no additional randomization needed when moving discard to deck.

In combat, resource cards that draw new cards are the primary mechanism to avoid hand depletion. Adjust the card gain from relevant resource cards to avoid card depletion — do NOT change deck or hand sizes for this purpose. (Non-combat disciplines auto-draw 1 card per play, so hand depletion is not a concern for them.)

### Healing and Stamina Recovery

Moderate healing and stamina recovery cards exist in the resource deck, allowing small amounts of recovery during combat. However, the main healing and stamina gain should come from resting encounters. Combat resource cards provide supplemental recovery only.

## Strict Invariants

These are non-negotiable constraints that all combat configs MUST satisfy:

1. **Deck size**: Every player combat deck (Attack, Defence, Resource) must contain exactly **50 cards** (deck + hand combined).
2. **Hand size**: The starting hand size is **5 cards per deck**, drawn from the deck at game start.
3. **Tier value ordering**: `free_max < stamina_cost_min < health_cost_min` for all effect types. Cost cards must always outperform free cards in raw effect value.
4. **Enemy deck size**: Enemy decks should match player deck size (50 cards each).

## Card Cost Distribution Guidelines

All combat decks follow a three-tier cost distribution that creates a risk/reward spectrum:

### Abundance Tiers

1. **Free cards** (no cost) — the most abundant in every deck (majority). These are the bread-and-butter plays with moderate effect values.
2. **Stamina-cost cards** — the second most common tier (moderate fraction). These cost Stamina but always outperform free cards in raw effect value.
3. **Health-cost cards** — rare but devastating (small fraction). These sacrifice Health but deal enough damage to justify the cost most times.

### Design Intent by Deck

**Attack Deck**: Mix of free damage, stamina-cost damage (reliably stronger), and rare health-cost damage (strongest). The gap between free max and stamina-cost min is the primary differentiation lever.

**Defence Deck**: Roughly 50/50 split between shield cards and dodge cards. Both have free and stamina-cost tiers. Stamina-cost dodge should provide massive absorption that fully blocks typical enemy attacks.

**Resource Deck**: Split across multiple focuses — draw-only cards (most common), stamina recovery, healing, and combination cards. Draw cards prevent hand depletion; stamina cards fuel cost-card plays; heal cards provide minor HP sustain.

**Enemy Decks**: Uniform decks (one effect type per deck). Enemy damage, shield, and resource/draw cards. Enemy shield cap must be non-zero (a cap of 0 is a known bug).

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

*Simulation results belong in PR descriptions, not in this document. See PR history for specific run data.*

## General Design Principles

These principles apply across all disciplines. For full details, see `docs/design/vision.md`.

### Card Cost Distribution

- **Free cards** (no cost) should be the most common card type in every deck.
- **Stamina-cost cards** should be moderately common and always outperform free cards in raw effect value.
- **Health-cost cards** should be rare but powerful, always outperforming stamina-cost cards.
- This creates a risk/reward spectrum: safe low-output plays → moderate-cost moderate-output → high-risk high-output.

### Mutator Scope

Balance mutators (the agents implementing balance changes) **may** change within their discipline:
- Any CardEffect within combat (including suggesting new CardEffects as a last resort)
- Any Card within combat (including suggesting new Cards, but try without first)
- Any encounter within combat (including suggesting new encounters, but try without first)

Balance mutators **must NOT** change:
- Starting Health, Stamina, or any player starting tokens
- Health or Stamina after death
- Hand sizes (all must remain 5)
- Deck sizes (all must remain 50)
- Anything outside the combat discipline

### Deck and Hand Sizing

- All player deck hand sizes: **5** (controlled by per-deck MaxHand tokens)
- All player deck sizes: **50** (controlled by per-deck MaxDeck tokens)
- Do NOT change deck or hand sizes to fix balance issues — adjust card effects and encounter parameters instead.

## Tuning Tips

- **RNG Coupling**: Changing card counts changes the RNG state for the entire game. Results from different configs are not directly comparable per-encounter. Only aggregate metrics are meaningful.
- **Shield within combat**: Since shield expires at combat end, shield values can be somewhat higher per card without creating compounding advantages across encounters. The balance lever is the relationship between enemy damage per round and shield grant per round within a single combat.
- **Death spiral interaction**: After player death, scouting generates easier encounters (configurable reduction). This prevents compounding difficulty from making the game unwinnable.
- **Scouting mutation asymmetry**: Enemy HP scales fully with difficulty factor, but enemy card effects only scale probabilistically. This creates "HP sponge" encounters at high difficulty. Future work should address proportional scaling (see scouting balance).
