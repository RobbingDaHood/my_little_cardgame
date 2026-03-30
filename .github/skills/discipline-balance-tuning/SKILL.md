---
name: discipline-balance-tuning
description: Step-by-step workflow for tuning a single discipline's balance. Prevents common pitfalls by enforcing API inspection before code changes and providing iteration shortcuts.
---

# Discipline Balance Tuning Workflow

Use this skill when tuning balance for a specific gathering discipline (herbalism, mining, woodcutting, fishing). It builds on `balance-tuning-tips` with a concrete, ordered workflow.

**Load `balance-tuning-tips` first** — it has essential background on RNG coupling, config embedding, and cost systems.

## Phase 0: Setup (do not skip)

1. Create a worktree: `scripts/worktree-manage.sh add <discipline>-balance-tuning`
2. Read the authoritative balance doc: `docs/vision/balances/<discipline>_balance.md`
3. Record the exact tier targets, strategy definitions, and constraints before touching code.

## Phase 1: API Format Verification (MANDATORY — prevents the #1 time sink)

**Before writing or modifying ANY driver or strategy code**, run the API format inspector:

```bash
scripts/balance-api-inspect.sh <discipline>
```

This runs `cargo test --features simulation --test balance api_inspect_<discipline> -- --nocapture` and prints the actual JSON field names and nesting for:
- Card effect templates (`.kind.kind.effect_type` — note the double-nested `kind`)
- Concrete hand cards (`.kind.effects[0].effect_id`, `.kind.effects[0].rolled_costs[].token_type`)
- Encounter state (`.encounter_state_type`, `.plant_hand`, `.outcome`)
- Player tokens (exact token name strings like `"Durability"` vs `"HerbalismDurability"`)

**Common pitfalls this prevents:**
- Token names differ between config (`"Durability"`) and runtime (`"HerbalismDurability"`)
- Card kind uses double nesting: outer `.kind.card_kind`, inner `.kind.kind.effect_type`
- Match mode lives at `.kind.kind.match_mode`, NOT `.kind.effect_type`
- `/actions/possible` returns placeholder `card_id: 0` — must query hand directly

**Cross-check your driver code** against the inspector output. If any field path in `get_playable_*_cards()` or `build_effect_map()` doesn't match, fix it BEFORE running simulations. Skipping this step has historically cost 40%+ of session time on debugging.

## Phase 2: Baseline

Run the existing simulation to establish baseline numbers:

```bash
scripts/balance-sim.sh <discipline>
```

This runs `cargo test --features simulation --test balance <discipline>_balance_simulation -- --nocapture` and extracts the JSON report.

Record baseline: strategy name, yield/dur, win rate, rounds/encounter. Compare against balance doc targets.

## Phase 3: Config Tuning

The primary tuning lever is `configurations/<discipline>/cards.json`. Key design principles:

1. **Create cost tiers**: Free cards (durability-only) vs costly cards (stamina/health + durability). This creates strategic differentiation.
2. **Diverse match modes**: Multiple card effect types so strategies can differentiate.
3. **Reward calibration**: Adjust reward amounts to bring the weakest strategy (Random) above Tier 1 minimum.

After each config change, run `scripts/balance-sim.sh <discipline>` to check directional impact. Config changes trigger a rebuild (~11s) via `include_str!()`.

## Phase 4: Strategy Development

When adding a new strategy:

1. Create `tests/balance/<discipline>/strategies/<name>.rs`
2. Add `pub mod <name>;` to `tests/balance/<discipline>/strategies/mod.rs`
3. Import and register in `tests/balance/<discipline>/<discipline>_test.rs`
4. Add yield target in `tests/balance/<discipline>/output.rs`

**Strategy differentiation tips:**
- Tier 1 strategies should NOT read encounter state (Random, Greedy, Conservative)
- Tier 2 strategies MUST read encounter state and make decisions based on it
- The non-yield tactician must win through play decisions only (no yield-boosting effects)
- The yield-optimizer can use any means (scouting, optimal sequences, yield effects)
- 2-step look-ahead adds marginal benefit when single-card greedy is already near-optimal

## Phase 5: Validation

```bash
make check              # fmt, clippy, tests, coverage (MUST pass before commit)
make balance-check      # all discipline simulations (~7 min)
```

## Phase 6: Commit and PR

Each commit must pass `make check`. Use `BREAKING:` prefix if config changes alter encounter structure. Include balance results table in PR body.

## Quick Reference: Scripts

| Script | Purpose |
|--------|---------|
| `scripts/balance-api-inspect.sh <disc>` | Dump actual API JSON formats (run FIRST) |
| `scripts/balance-sim.sh <disc>` | Run single discipline simulation |
| `scripts/balance-sim.sh` | Run all discipline simulations |
| `make check` | Full validation (fmt, clippy, tests, coverage) |
| `make balance-check` | All balance simulations |

## Iteration Speed Tips

- During exploration, use 3 games × 20 encounters for ~8-12s runs (vs 10+ games for full validation)
- Config-only changes rebuild in ~11s; strategy code changes may take longer
- Run `scripts/balance-sim.sh <discipline>` after EVERY change — fast feedback beats batch debugging
- If a strategy performs unexpectedly, add temporary `eprintln!` tracing behind an env var check, then remove before committing
