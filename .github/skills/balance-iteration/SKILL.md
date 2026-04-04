---
name: balance-iteration
description: Step-by-step balance tuning iteration for a single discipline. Load this skill when doing config tuning to follow the optimal workflow and avoid common time sinks.
---

# Balance Iteration Workflow

Follow this workflow when tuning balance configs for a specific discipline. The goal is to reach documented targets in the fewest wall-clock minutes.

## Before Starting

1. **Load `balance-tuning-tips` skill** — read all tips before touching config
2. **Read the discipline's balance doc** — e.g., `docs/vision/balances/mining_balance.md`
3. **Run baseline** — `make balance-<discipline>` (e.g., `make balance-mining`) to establish current numbers
4. **Compute per-round economics** — before changing anything, calculate:
   - Expected durability cost per round (weighted ore card distribution)
   - Expected yield per round at various key-token levels (e.g., light=200, 170, 140)
   - Which cards produce yield vs which are utility (light, stamina, etc.)
   - Verify that utility cards pay back their round cost (they MUST — only insight cards are exempt)
5. **Update the discipline's balance doc** — if new mechanics or rules are being introduced, update the balance document BEFORE running the balancing step so the doc reflects the new rules

## Iteration Cycle

**ALWAYS use parallel worktrees for exploration** (see `parallel-balance-tuning` skill).

For each round of iteration:

### 1. Design 2-3 config variants

Spread across the parameter space. Common levers:
- **Ore/enemy cost values** (durability damage, light loss, health damage)
- **Player card power ranges** (min/max on yield-producing effects)
- **Utility card values** (light gain, stamina gain)
- **Deck composition** (ratio of power / utility / cost cards)
- **Initial encounter values** (initial light level, etc.)

### 2. Launch parallel agents

Each agent gets its own worktree, edits the config, and runs:

```bash
scripts/balance-quick.sh <discipline>
```

**NOT** `make balance-check` — that runs all 7 tests and wastes ~170s on unrelated disciplines.

### 3. Aggregate results

Collect yield/dur from each variant. Identify which parameter direction improves metrics.

### 4. Repeat or finalize

If targets are met → run `make balance-check` (all disciplines) then `make check` (full validation).
If not → design next round of variants based on what you learned.

## Strategy Tuning

If config changes alone don't reach targets, the strategy implementations may need work:

1. **Trace the driver** — add `eprintln!` in the strategy's `choose_card` method to see per-round decisions
2. **Check for traps** — does the strategy waste rounds on 0-yield cards? (See "Round Cost Is Fixed" in balance-tuning-tips)
3. **Test conclude thresholds** — the strongest lever for tier-2 strategies is usually conclude timing, not card selection

## Common Pitfalls

- ❌ Running `make balance-check` every iteration (runs ALL disciplines, ~190s)
- ❌ Sequential iteration (edit → test → read → think → edit → test) — use parallel worktrees
- ❌ Assuming 1 ore card = 1 effect (many have compound effects)
- ❌ Utility cards that don't pay back their round cost (they MUST — only insight cards are exempt)
- ❌ Not computing per-round economics before starting
- ❌ Not updating the balance doc before running the balancing step

## Time Budget

| Step | Expected Time |
|------|--------------|
| Baseline + economics analysis | ~5 min |
| Per iteration round (parallel) | ~2 min (build + test) |
| Per iteration round (sequential) | ~6 min (3× slower) |
| Final validation (`make balance-check` + `make check`) | ~5 min |
| Doc updates + commit + PR | ~5 min |

Target: 3-5 iteration rounds × 2 min each = **20-25 min total** for a discipline tuning session.
