---
name: balance-tuning-tips
description: Key learnings and tips for balance simulation tuning sessions. Reference this skill before starting any balance work to avoid common pitfalls.
---

# Balance Tuning Tips

Lessons learned from B2.1 combat simulation tuning. Read before starting any balance tuning session.

For combat-specific balancing guidance, see `docs/vision/balances/combat_balance.md`.
For scouting-specific balancing guidance, see `docs/vision/balances/scouting_balance.md`.
The tuning phases are defined in the `parallel-balance-tuning` skill.

## RNG Coupling Warning

Changing card counts changes the RNG state for the entire game. This means:
- Results from different configs are **not directly comparable** on a per-encounter basis
- Only **aggregate metrics** (average streak, win rate over many games) are meaningful for comparison
- Each config must be evaluated independently with sufficient sample size

## Bug Prevention: Pre-Investigate API Formats

Before writing any extraction or analysis code, launch an explore agent to examine the actual JSON response formats:

```
task(agent_type="explore", mode="background",
     prompt="Examine JSON responses from /encounter, /library/cards, /actions/possible,
             /player/tokens. Document exact field names, nesting, and key formats.")
```

Common pitfalls discovered in B2.1:
- Token keys are `"Health"` not `"Health:PersistentCounter"`
- `card.kind` uses tagged enum serialization (check `encounter_state_type` field)
- Actions in `/actions/possible` use specific enum variant names

## Compile-Time Config Embedding

Configs are embedded via `include_str!()` in `src/library/config_loader.rs`. Touching any file under `configurations/` triggers a full crate rebuild (~11s). For rapid iteration:
- Use runtime config loading (behind `--features simulation`) when available
- Or use `GameState::new_from_json()` with custom JSON strings in tests

## Reduced Simulation Size for Exploration

During the exploration phase, rough directional signals are sufficient:

| Config | Games | Encounters | Runtime | Quality |
|--------|-------|-----------|---------|---------|
| Full | 10+ | 50 | ~44s | Good |
| Exploration | 3 | 20 | ~8-12s | Rough |
| Quick-check | 1 | 10 | ~2-3s | Directional |

Only run full simulations for final validation.

## Card Persistence Across Encounters

Cards are **never reset** between encounters. When a card is drawn from the deck and there are no more cards to draw, the full discard pile is moved into the deck. Because cards are randomized when drawing from deck to hand, there is no additional randomization needed when moving discard to deck.

In combat, adjust the card gain from relevant resource cards to avoid card depletion — do NOT change deck or hand sizes for this purpose.

## Cost System

- **Templates** (`CardEffectCost`): percentage-based costs (`min_percent`/`max_percent`)
- **Concrete cards** (`ConcreteEffectCost`): always absolute values (`amount: u32`)
- All costs are pre-computed at roll time — no per-play randomness
- `is_absolute: true` means the rolled value IS the cost (not a percentage of gain)
