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

Create 3 worktrees for parallel exploration:

```bash
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

After 2-3 rounds of narrowing, apply the best config to the main repo. Run full validation:

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
4. **3 rounds × 3 variants typically sufficient** to converge on targets
5. **CPU: 4 cores available** — 3 parallel builds/tests run at ~75% efficiency

## Cleanup

```bash
./scripts/worktree-manage.sh remove wt-a
./scripts/worktree-manage.sh remove wt-b
./scripts/worktree-manage.sh remove wt-c
```

## Balance Targets (Combat)

| Strategy | Min Streak | Max Streak |
|----------|-----------|-----------|
| Random | 3.5 | 8.0 |
| Greedy | 3.0 | 7.0 |
| Conservative | 2.5 | 6.0 |
| Tactician | 8.0 | 18.0 |

All strategies should average ≥3.0 rounds per encounter.
