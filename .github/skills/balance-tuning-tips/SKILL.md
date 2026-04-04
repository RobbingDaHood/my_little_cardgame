---
name: balance-tuning-tips
description: Key learnings and tips for balance simulation tuning sessions. Reference this skill before starting any balance work to avoid common pitfalls.
---

# Balance Tuning Tips

Lessons learned from balance simulation tuning. Read before starting any balance tuning session.

For combat-specific balancing guidance, see `docs/vision/balances/combat_balance.md`.
For scouting-specific balancing guidance, see `docs/vision/balances/scouting_balance.md`.
For mining-specific balancing guidance, see `docs/vision/balances/mining_balance.md`.
For herbalism-specific balancing guidance, see `docs/vision/balances/herbalism_balance.md`.
For woodcutting-specific balancing guidance, see `docs/vision/balances/woodcutting_balance.md`.
For fishing-specific balancing guidance, see `docs/vision/balances/fishing_balance.md`.
The tuning phases are defined in the `parallel-balance-tuning` skill.

## Quick Commands

Run a single discipline's balance simulation during iteration — **do NOT run all 7 tests each cycle**:

```bash
make balance-mining        # ~12s — mining only
make balance-combat        # combat only
make balance-herbalism     # herbalism only
make balance-woodcutting   # woodcutting only
make balance-fishing       # fishing only
make balance-check         # ~190s — all disciplines (final validation only!)
```

Or directly: `scripts/balance-quick.sh <discipline>`

## Iteration Workflow

**CRITICAL: Use parallel exploration, not sequential iteration.**

The `parallel-balance-tuning` skill exists for a reason. Each config→build→test cycle takes ~20s. Sequential iteration (edit, test, read, think, repeat) wastes most of the session. Instead:

1. **Phase 1**: Launch 3 config variants in parallel worktrees (broad sweep)
2. **Phase 2**: Narrow based on results, launch 3 more variants
3. **Phase 3**: Fine-tune the winner

A 3-iteration sequential session takes ~60s of wall time per round. 3 parallel variants take ~20s total per round.

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

## Gathering-Specific Lessons (Mining, Woodcutting, Herbalism, Fishing)

### Utility Cards Must Pay Back Their Round Cost

In gathering encounters, an environment card plays every round regardless of what the player plays. This means every round has a **fixed durability cost**. A card that doesn't directly produce yield must generate enough benefit in subsequent rounds to exceed this round cost.

**Design principle**: Every card type must have a situation where playing it is optimal. Utility cards (light boost, stamina gain, etc.) must be tuned so their benefit exceeds the durability cost of the round they consume. The only exception is **insight cards**, which intentionally increase difficulty and are meant to be strategically undesirable.

If a utility card type is never worth playing, the mechanic needs adjustment — see the discipline-specific balance document for design directions (e.g., `mining_balance.md` "Light Level as Ramping Yields" section).

### Conclude Timing Dominates Strategy

The strongest gathering strategies tend to converge on: **play one high-value card at peak conditions, then conclude immediately**. This suggests the conclude-timing mechanic is the dominant lever.

If multiple Tier-2 strategies converge on the same behavior (~0.5 rounds/encounter), the config may need mechanics that reward multi-round play (e.g., ramping yields, reduced costs for consecutive plays).
