# Scouting Balance

This document contains scouting-specific balancing information. It is the authoritative reference for scouting difficulty mechanics, configuration, and known issues.

## Difficulty Delta System

Scouting generates replacement encounters by mutating the source encounter with a difficulty delta. The delta is a multiplier offset: `factor = 1.0 + delta`.

### Configuration Parameters

Located in `configurations/general/game_rules.json` under `"scouting"`:

| Parameter | Description | Current Value |
|-----------|-------------|---------------|
| `choice_count` | Number of scouting choices offered | 3 |
| `difficulty_delta_min` | Minimum difficulty delta | -0.05 |
| `difficulty_delta_max` | Maximum difficulty delta | 0.4 |
| `difficulty_delta_min_separation` | Minimum separation between deltas | 0.1 |
| `mutation_fraction` | Fraction of deck cards mutated | 0.2 |
| `mutation_scale_probability` | Probability of scale mutation | 0.5 |
| `mutation_redistribute_probability` | Probability of redistribute mutation | 0.3 |
| `death_difficulty_reduction_min` | Min reduction after death | -0.25 |
| `death_difficulty_reduction_max` | Max reduction after death | -0.05 |

### Death Difficulty Reduction

When a player dies, the next scouting phase generates encounters that are easier than the killing encounter. The reduction range is configurable via `death_difficulty_reduction_min` and `death_difficulty_reduction_max`. This prevents the death spiral from making the game unwinnable and creates a "catch-up" mechanic.

### Known Issues and Pitfalls

- **Infinite loop risk**: `difficulty_delta_min_separation` must be less than the total delta range (`delta_max - delta_min`). A zero-width range causes infinite loops in encounter generation. Ensure `(delta_max - delta_min) > (choice_count - 1) × min_separation` with margin.
- **Current values**: delta_min=-0.05, delta_max=0.4, min_separation=0.1

## Mutation System (Current: Simple Approach)

The current scouting mutation system is intentionally simple. It creates an asymmetry between token scaling and card effect scaling:

- **Token scaling (initial_tokens)**: Scales 100% with the difficulty factor. Enemy HP, starting resources, etc. all scale proportionally.
- **Card effect scaling**: Only ~10% of cards are affected per step (`mutation_fraction × scale_probability = 0.20 × 0.50 = 10%`). This means enemy damage, shield, and other card effects lag behind HP scaling.

This asymmetry creates "HP sponge" encounters at high difficulty — long fights with moderate danger. The current system works as a baseline but does not fully capture encounter difficulty.

### Future Vision: Per-Encounter Difficulty Adjustment

The goal is to evolve scouting so that all aspects of an encounter are considered when calculating the difficulty of new encounters. The future approach for each scouted encounter:

1. **Roll card mutations** within a ~10-20% limit so the new encounter still feels familiar to the player.
2. **Roll the difficulty delta** from the configured range.
3. **Adjust all initial tokens** (enemy HP, resources, etc.) so the encounter follows the new difficulty level proportionally.

This keeps encounters recognizable (small card mutations) while ensuring difficulty scales holistically through token adjustments. It should be straightforward to implement and not overly difficult to balance.

### Additional Future Work

- **Per-token-type scaling**: Different tokens could scale at different rates (e.g., enemy HP scales at 80%, enemy damage at 100%) for finer balance control.
- **Scouting difficulty reset granularity**: The death difficulty reduction could be more nuanced — e.g., scaling with the number of consecutive deaths, or with the magnitude of the difficulty gap.

## General Design Principles

These principles apply across all disciplines. For full details, see `docs/design/vision.md`.

### Mutator Scope

Balance mutators (the agents implementing balance changes) **may** change within scouting:
- Scouting configuration parameters (delta ranges, mutation fractions, probabilities)
- Death difficulty reduction parameters
- Choice count

Balance mutators **must NOT** change:
- Starting Health, Stamina, or any player starting tokens
- Health or Stamina after death
- Hand sizes (all must remain 5)
- Deck sizes (all must remain 50)
- Anything outside the scouting discipline

### Deck and Hand Sizing

- All player deck hand sizes: **5** (controlled by per-deck MaxHand tokens)
- All player deck sizes: **50** (controlled by per-deck MaxDeck tokens)
- Do NOT change deck or hand sizes to fix balance issues — adjust card effects and encounter parameters instead.
