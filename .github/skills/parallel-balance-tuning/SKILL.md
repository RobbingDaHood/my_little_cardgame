---
name: parallel-balance-tuning
description: Parallel balance tuning workflow using git worktrees and background agents. Use this skill when iterating on game balance configs to test multiple variants simultaneously.
---

# Parallel Balance Tuning

This skill accelerates balance config tuning by running multiple config variants in parallel using git worktrees and background agents.

## Prerequisites

- The `simulation` feature flag must be available (`cargo test --features simulation`)
- `scripts/worktree-manage.sh` manages worktrees

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

Design 3 config variants spanning the parameter space. Launch 3 background `general-purpose` agents, each operating in a separate worktree:

```
task(agent_type="general-purpose", mode="background", name="tuning-variant-a",
     prompt="In worktree /path/to/wt-a, edit configurations/combat/cards.json
             to set [specific changes]. Run: cargo test --features simulation --test balance -- --nocapture
             Report the overall_avg_streak for each strategy and whether targets are met.")

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
make balance-check
make check
```

## Reduced Simulation Size for Exploration

During exploration, use smaller simulations for faster feedback. Only run full simulations for final validation. If `SimulationConfig` supports it, reduce `games_per_strategy` and `encounters_per_game` during exploration rounds.

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
