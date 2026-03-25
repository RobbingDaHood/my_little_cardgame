# Suggested Updates to vision.md and roadmap.md

Based on the B2.1 combat simulation runner implementation and its findings.

---

## vision.md Suggestions

### 1. Update "Tuning pipeline and instrumentation" section (~line 536)

**Current text:**
> Headless Monte Carlo simulation (PRIMARY): Scripted strategy bots (random, greedy, conservative, discipline-specific) play thousands of games via the REST API at CPU speed, producing statistically significant win-rate data per encounter type. Multiple server instances can run in parallel on different ports.

**Suggested replacement:**
> Headless Monte Carlo simulation (PRIMARY): Scripted strategy bots (random, greedy, conservative, discipline-specific) play thousands of games at CPU speed, producing statistically significant win-rate data per encounter type. The primary runner is Rust-based, using `rocket::local::blocking::Client` for in-process testing (no HTTP server needed, maximum speed). External Python/nushell scripts can also drive the REST API for ad-hoc analysis. Feature-gated under `--features simulation` so normal `cargo test` is unaffected.

### 2. Add "Regression checks" detail (~line 540)

**Current text:**
> Regression checks: `make balance-check` target runs quick simulations and asserts win rates stay within documented target ranges

**Suggested replacement:**
> Regression checks: `make balance-check` runs the Rust simulation suite (`cargo test --features simulation --test balance`) and asserts win rates stay within documented target ranges. Currently exercises combat encounters only; will expand to all disciplines as B2.x runners are added.

### 3. Add simulation findings note (new subsection after target win rates)

Add a subsection documenting baseline findings:
> **Baseline findings (B2.1):** Initial 1000-game simulation (seed 42, 3 strategies × 20 max encounters) shows ~99% combat win rate across all strategies with ~3 encounters per game before stamina depletion. This confirms combat is significantly too easy relative to the ~50% greedy / ~30% random targets. Key observations:
> - All strategies perform nearly identically (~99% win rate), indicating card selection has minimal impact on combat outcome
> - Games terminate after ~3 encounters due to stamina depletion, not player death
> - Deaths are rare (~3% of games) and occur from accumulated damage, not tactical failure
> - Combat rebalancing should focus on: enemy damage scaling, stamina economy, and card differentiation

---

## roadmap.md Suggestions

### 1. Insert B2.1 between B2 and B3

Add new step:

> **B2.1) Combat simulation runner (in-process, Rust)**
>
> **Goal**: Build the first headless simulation runner focused on combat encounters, establishing the test infrastructure and strategy bot patterns for all future discipline runners.
>
> **Description**: Rust integration tests under `tests/balance/` gated by `simulation` feature flag. Three strategy bots (random, greedy, conservative) drive combat encounters via public API only (`/actions/possible`, `/encounter`, `/player/tokens`, `/library/cards`). Runner plays 1000 games per strategy with deterministic seeds, outputs JSON report with win rates and token flows, asserts against vision.md targets (±10%).
>
> **Key components**:
> - `tests/balance/strategies/` — Strategy trait + 3 implementations (random, greedy, conservative)
> - `tests/balance/game_driver.rs` — Single-game combat loop driver
> - `tests/balance/runner.rs` — Multi-game orchestration and aggregation
> - `tests/balance/output.rs` — JSON report formatting and assertion checking
> - `tests/balance/combat.rs` — Combat balance test (1000 games × 3 strategies)
> - `Makefile` target: `make balance-check`
>
> **Playable acceptance**: `make balance-check` completes, outputs JSON report to stdout, and reports combat win rates per strategy. Test currently FAILS because combat is too easy (~99% win rate vs 20-50% targets) — this is a valid balance signal, not a test infrastructure bug.
>
> **Baseline findings**: All strategies achieve ~99% win rate. Games last ~3 combat encounters before stamina depletion. Strategy differentiation is minimal. These findings inform the priority of balance changes in B2 (manual config pass).

### 2. Add B2.2–B2.9 substeps for future discipline runners

After B2.1, add placeholders:

> **B2.2) Mining simulation runner** — Extend balance test infrastructure with mining-specific strategy logic. Reuses Strategy trait and GameDriver from B2.1.
>
> **B2.3) Herbalism simulation runner**
>
> **B2.4) Woodcutting simulation runner**
>
> **B2.5) Fishing simulation runner**
>
> **B2.6) Rest simulation runner**
>
> **B2.7) Crafting simulation runner**
>
> **B2.8) Research simulation runner**
>
> **B2.9) Whole-game balance simulation** — Cross-discipline interactions, resource flows, death frequency, progression pacing. Runs full game sessions (all encounter types) and validates overall progression curve.

### 3. Update B3 tooling references

**Current text (B3):**
> `tools/balance/runner.py` — main simulation runner
> `tools/balance/strategies.py` — strategy implementations per discipline
> Uses only `requests` library

**Suggested replacement:**
> The per-discipline simulation runners from B2.x replace the originally planned Python scripts. All runners are Rust integration tests under `tests/balance/` using `rocket::local::blocking::Client` for in-process execution. External Python/nushell scripts may still be used for ad-hoc analysis of the JSON output.

Specifically, B3 should focus on **analysis and iteration** rather than runner construction:
> **B3) Balance analysis and iteration**
> - Analyze B2.x simulation results to identify imbalances
> - Propose and test config changes using the simulation infrastructure
> - Iterate until win rates meet vision.md targets

### 4. Remove worktree-isolation requirement from B3

The B2.x Rust in-process approach eliminates the need for worktree isolation. All discipline runners share the same `tests/balance/` directory and production configs. Config modifications for testing can be done through the `create_test_client_from_json()` helper or by adjusting test parameters.
