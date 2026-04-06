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

## Quick Reference: Balance Sim Commands

Run a single discipline quickly during exploration (no recompile needed):

```bash
# Quick: 1 game × 10 encounters (~2-5s) — directional signal only
scripts/balance-sim.sh woodcutting --quick

# Explore: 3 games × 20 encounters (~8-15s) — rough signal for iteration
scripts/balance-sim.sh woodcutting --explore
# Or equivalently:
make balance-quick D=woodcutting

# Full: test-defined values (~40-500s) — final validation only
scripts/balance-sim.sh woodcutting --full

# Custom: set exact values via env vars
SIM_GAMES=5 SIM_ENCOUNTERS=30 scripts/balance-sim.sh mining

# All disciplines full validation:
make balance-check
```

**Critical**: Use `--quick` or `--explore` during tuning iteration. Only run `--full` for final validation. Each config change triggers a ~11s rebuild; don't compound it with a ~500s full sim.

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

**Use env-var overrides** to switch sim size without recompiling:

```bash
# Exploration: rough directional signal
SIM_GAMES=3 SIM_ENCOUNTERS=20 scripts/balance-sim.sh woodcutting
# Quick-check: fastest possible signal
SIM_GAMES=1 SIM_ENCOUNTERS=10 scripts/balance-sim.sh woodcutting
# Full: final validation (uses test-defined defaults)
scripts/balance-sim.sh woodcutting --full
```

| Config | Games | Encounters | Runtime | Quality |
|--------|-------|-----------|---------|---------|
| Full | 10+ | 50 | ~44s | Good |
| Exploration | 3 | 20 | ~8-12s | Rough |
| Quick-check | 1 | 10 | ~2-3s | Directional |

Only run full simulations for final validation.

## Card Persistence and Depletion Across Encounters

Cards are **never reset** between encounters. When a card is drawn from the deck and there are no more cards to draw, the full discard pile is moved into the deck. Because cards are randomized when drawing from deck to hand, there is no additional randomization needed when moving discard to deck.

### ⚠️ Card Depletion in Gathering Disciplines (Critical)

In gathering disciplines (woodcutting, mining, herbalism, fishing), the **last play** of each encounter does NOT draw a replacement card. This causes a net loss of ~1 hand card per encounter. With a starting hand of 5 cards, the hand empties after ~5 encounters, and all subsequent encounters conclude immediately with 0 plays (0 yield, 0 durability cost).

**This is NOT a bug** — it's the intended session-arc mechanic. Key implications:
- The yield/durability ratio is driven by the first ~5 "active" encounters, not the full session
- 0-play encounters contribute 0/0, which doesn't affect the ratio
- Durability budget (e.g., 10000 WoodcuttingDurability) far exceeds what cards can spend — the true session limiter is card supply, not durability
- Do NOT waste time adding diagnostic code to investigate why encounters are short — check hand card count first

In combat, adjust the card gain from relevant resource cards to avoid card depletion — do NOT change deck or hand sizes for this purpose.

## Systematic Tuning Recipe

Follow this order to avoid wasted iteration:

1. **Read the discipline balance doc** (`docs/vision/balances/<discipline>_balance.md`) — understand tier targets, strategy requirements, and mechanical levers.

2. **Run baseline** with `--explore` mode — establish current numbers before changing anything.

3. **Tune max_plays first** (if the discipline uses it) — this is the strongest lever for strategy differentiation. Lower max_plays = random play gets worse outcomes, strategic play is rewarded more.

4. **Tune costs second** — adjust durability/stamina/health costs to bring the yield/durability ratio into the target range. Work in large steps first (2-3x changes), then fine-tune.

5. **Tune multipliers/rewards last** — adjust pattern multipliers or base rewards to separate tiers. Only needed if cost tuning alone doesn't create enough tier differentiation.

6. **Validate with `--full`** only after `--explore` shows targets are met.

7. **Run `make check`** to ensure scenario tests still pass (config changes may break hardcoded assertions).

## Cost System

- **Templates** (`CardEffectCost`): percentage-based costs (`min_percent`/`max_percent`)
- **Concrete cards** (`ConcreteEffectCost`): always absolute values (`amount: u32`)
- All costs are pre-computed at roll time — no per-play randomness
- `is_absolute: true` means the rolled value IS the cost (not a percentage of gain)
