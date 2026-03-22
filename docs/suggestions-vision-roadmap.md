# Suggestions for vision.md and roadmap.md

Based on discoveries during the B2.1 combat simulation runner implementation.

## vision.md suggestions

### 1. Add shield carryover as an explicit design principle
Shield (PersistentCounter) carries across encounters and is the **dominant mechanic** determining long-term combat survival. Strategies that efficiently build shield between combats gain compounding advantages. This should be called out explicitly in the balancing section as a first-class design lever — it's more impactful than per-combat damage/defense tuning.

### 2. Document card persistence across encounters
Cards are never reset between encounters. This means card depletion over a full game session is a critical balancing dimension. If basic cards run out, ALL strategies converge regardless of intelligence. Vision should acknowledge card persistence as a design choice and describe its balancing implications (e.g., deck sizes must be generous enough that strategy differentiation is maintained across a full session).

### 3. Add strategy tier definitions
The vision currently mentions "multiple viable strategies" but doesn't define tiers. Suggest adding:
- **Simple tier** (random, greedy, conservative): No encounter-state awareness. Picks cards based on cost or value only.
- **Intermediate tier** (tactician): Reads enemy hand/token state. Adapts card selection per encounter.
- **Advanced tier** (future: meta-strategist): Manages resources across encounters. Plans card usage over multiple combats.
Each tier should have target streak ranges to make balancing goals concrete.

### 4. Clarify cost system semantics
The cost percentage on a card is a percentage of the **effect's rolled value**, not of the player's current HP/resource pool. This is non-obvious and has major balancing implications — a 50% cost on a 1000-damage attack costs 500 HP, while a 1% cost costs only 10 HP. Vision should document this clearly so future card designers understand cost impact.

## roadmap.md suggestions

### 1. Replace Python tooling references with Rust
Several B4/B5/B8 sections still reference `tools/balance/analyze.py`, `tools/balance/llm_analyze.py`, `tools/balance/auto_balance.py`. Now that B2.1 established Rust integration tests as the simulation approach, consider updating these references or noting that the implementation language may differ from the original plan.

### 2. Simplify worktree-based approach
The original B3 plan called for per-discipline git worktrees with separate server instances. B2.1 showed that in-process `rocket::local::blocking::Client` eliminates the need for running servers, managing ports, or isolating worktrees. The worktree approach adds complexity with little benefit when tests run in-process. Consider simplifying B3-B6 to use branch-based development with shared test infrastructure instead of worktree isolation.

### 3. Add B2.1 learnings as B3 guidance
Key tuning lessons from combat simulation that apply to all disciplines:
- **RNG coupling**: Changing card counts changes the RNG state, invalidating before/after comparisons across config changes. Each config must be evaluated independently.
- **Compile-time config embedding**: Configs are loaded via `include_str!` at compile time, so config changes affect ALL tests (not just balance tests). Balance-specific overrides require custom `GameState::new_from_json()`.
- **Scouting interaction**: Balance tests that use the full game loop must handle scouting encounters, which have their own config sensitivity (e.g., `difficulty_delta_min_separation` with a zero delta range causes infinite loops).

### 4. Document `make balance-check` as partially implemented
`make balance-check` already exists and runs combat simulation tests. Roadmap B9 still describes it as "(Future)" — update to reflect that combat balance regression is already operational, with future disciplines to be added incrementally.

### 5. Add a "balance test architecture" section
The test infrastructure in `tests/balance/` has a clear layered design that should be documented:
- `game_driver.rs` — generic game loop driver (discipline-agnostic)
- `runner.rs` — simulation runner (parallel game execution, result aggregation)
- `output.rs` — report formatting and assertion framework
- `strategies/mod.rs` — Strategy trait definition
- `<discipline>/` — per-discipline test, driver, output, and strategies

This architecture should be documented so future discipline runner authors follow the established pattern.
