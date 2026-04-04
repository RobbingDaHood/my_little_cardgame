---
name: balance-iteration
description: Step-by-step balance tuning iteration using parallel worktrees and background agents. Load this skill when doing config tuning to follow the optimal workflow and avoid common time sinks.
---

# Balance Iteration Workflow

Follow this workflow when tuning balance configs for a specific discipline. The goal is to reach documented targets in the fewest wall-clock minutes.

## Prerequisites

- The `simulation` feature flag must be available (`cargo test --features simulation`)
- `scripts/worktree-manage.sh` manages worktrees
- `scripts/balance-quick.sh` runs a single discipline's sim

## Before Starting

1. **Rebase on the development branch** — `git fetch origin && git rebase origin/main` to ensure you're working on the latest code. Resolve any conflicts before proceeding.
2. **Load `balance-tuning-tips` skill** — read all tips before touching config
3. **Read the discipline's balance doc** — e.g., `docs/vision/balances/mining_balance.md`
4. **Run baseline** — `make balance-<discipline>` (e.g., `make balance-mining`) to establish current numbers
5. **Compute per-round economics** — before changing anything, calculate:
   - Expected durability cost per round (weighted ore card distribution)
   - Expected yield per round at various key-token levels (e.g., light=200, 170, 140)
   - Which cards produce yield vs which are utility (light, stamina, etc.)
   - Verify that utility cards pay back their round cost (they MUST — only insight cards are exempt)
6. **Update the discipline's balance doc** — if new mechanics or rules are being introduced, update the balance document BEFORE running the balancing step so the doc reflects the new rules

## Phase 0: Setup Worktrees

Create worktrees for parallel exploration. Ensure worktrees do not already exist before creating — do NOT delete existing worktrees, create new ones with different names instead:

```bash
# Check existing worktrees first
./scripts/worktree-manage.sh list

# Create worktrees with unique names (adjust names if already taken)
./scripts/worktree-manage.sh add wt-a
./scripts/worktree-manage.sh add wt-b
./scripts/worktree-manage.sh add wt-c
```

Pre-build all worktrees in parallel to warm caches:

```bash
for wt in wt-a wt-b wt-c; do
  (cd "$(git rev-parse --git-common-dir)/../../my_little_cardgames/$wt" && cargo build --tests --features simulation) &
done
wait
```

## Phase 1: Broad Parameter Sweep

Design 3 config variants spanning the parameter space. Common levers:
- **Ore/enemy cost values** (durability damage, light loss, health damage)
- **Player card power ranges** (min/max on yield-producing effects)
- **Utility card values** (light gain, stamina gain)
- **Deck composition** (ratio of power / utility / cost cards)
- **Initial encounter values** (initial light level, etc.)

Launch 3 background `general-purpose` agents, each in a separate worktree:

```
task(agent_type="general-purpose", mode="background", name="tuning-variant-a",
     prompt="In worktree /path/to/wt-a, edit configurations/<discipline>/cards.json
             to set [specific changes]. Run: scripts/balance-quick.sh <discipline>
             Report the yield/dur for each strategy and whether targets are met.")

task(agent_type="general-purpose", mode="background", name="tuning-variant-b",
     prompt="In worktree /path/to/wt-b, edit [different changes]...same instructions...")

task(agent_type="general-purpose", mode="background", name="tuning-variant-c",
     prompt="In worktree /path/to/wt-c, edit [different changes]...same instructions...")
```

Wait for all 3 to complete, then aggregate results.

## Phase 2: Narrow Search

Based on Phase 1 results, identify the best-performing region. Design 3 new variants within that narrower region. Reset worktrees and launch another round:

```bash
./scripts/worktree-manage.sh reset wt-a
./scripts/worktree-manage.sh reset wt-b
./scripts/worktree-manage.sh reset wt-c
```

## Phase 3: Fine-tune and Validate

Keep iterating — do not limit to a fixed number of rounds. After each round:

1. Analyze results and identify the most promising direction
2. Consider whether the current path could ever reach the goals — if not, note it as a possibly blind path and start from a new broad approach
3. Keep track of ALL results so far and keep exploring the most promising lead
4. Regularly print a status summary for the user to observe progress

Apply the best config to the main repo when targets are met. Run full validation:

```bash
make balance-check   # all disciplines
make check           # fmt, clippy, tests, coverage
```

## Reduced Simulation Size for Exploration

During exploration, use smaller simulations for faster feedback. Only run full simulations for final validation. If `SimulationConfig` supports it, reduce `games_per_strategy` and `encounters_per_game` during exploration rounds.

## Strategy Tuning

If config changes alone don't reach targets, the strategy implementations may need work:

1. **Trace the driver** — add `eprintln!` in the strategy's `choose_card` method to see per-round decisions
2. **Check utility card viability** — does the strategy waste rounds on cards that don't pay back? (See "Utility Cards Must Pay Back Their Round Cost" in balance-tuning-tips)
3. **Test conclude thresholds** — the strongest lever for tier-2 strategies is usually conclude timing, not card selection

## Common Pitfalls

- ❌ Running `make balance-check` every iteration (runs ALL disciplines, ~190s)
- ❌ Sequential iteration (edit → test → read → think → edit → test) — use parallel worktrees
- ❌ Assuming 1 ore card = 1 effect (many have compound effects)
- ❌ Utility cards that don't pay back their round cost (they MUST — only insight cards are exempt)
- ❌ Not computing per-round economics before starting
- ❌ Not updating the balance doc before running the balancing step

## Key Principles

1. **Each agent gets its own worktree** — no filesystem conflicts
2. **Include ALL context in each agent prompt** — agents are stateless
3. **Compare aggregate metrics only** — RNG coupling makes per-encounter comparison invalid across different configs
4. **Keep iterating until targets are met** — track all results, explore the most promising lead, abandon blind paths early
5. **CPU: 4 cores available** — 3 parallel builds/tests run at ~75% efficiency
6. **Print regular status summaries** — so the user can observe progress

## Cleanup

```bash
./scripts/worktree-manage.sh remove wt-a
./scripts/worktree-manage.sh remove wt-b
./scripts/worktree-manage.sh remove wt-c
```

## Balance Targets

For detailed per-discipline balance targets, see the balance docs:
- Combat: `docs/vision/balances/combat_balance.md` — streak-based metrics, strategy hierarchy, win rate targets
- Mining: `docs/vision/balances/mining_balance.md` — yield per durability targets
- Herbalism: `docs/vision/balances/herbalism_balance.md` — yield per durability targets
- Woodcutting: `docs/vision/balances/woodcutting_balance.md` — yield per durability targets
- Fishing: `docs/vision/balances/fishing_balance.md` — yield per durability targets
- Scouting: `docs/vision/balances/scouting_balance.md` — difficulty delta targets

**General principle:** Combat balance uses consecutive win streaks; gathering discipline balance uses yield per durability. All gathering disciplines should produce roughly the same yield value per durability. Simple approaches should always be less rewarding than tactical approaches.

## Time Budget

| Step | Expected Time |
|------|--------------|
| Baseline + economics analysis | ~5 min |
| Per iteration round (parallel) | ~2 min (build + test) |
| Per iteration round (sequential) | ~6 min (3× slower) |
| Final validation (`make balance-check` + `make check`) | ~5 min |
| Doc updates + commit + PR | ~5 min |

Target: 3-5 iteration rounds × 2 min each = **20-25 min total** for a discipline tuning session.
