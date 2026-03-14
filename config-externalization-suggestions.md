# Vision & Roadmap Improvement Suggestions

Observations from implementing Roadmap Step 14 (Configuration Externalization).

## vision.md suggestions

1. **Add a "Configuration architecture" section** explaining the JSON config → compile-time embedding → Library initialization pipeline. This is now a core architectural pattern that affects how designers interact with the system.

2. **Update "Architecture and module layout"** to mention `configurations/` as a top-level directory alongside `src/`. Currently the module layout only describes Rust code, but the JSON configs are now equally important for understanding the system.

3. **Add designer workflow description**: The vision could benefit from describing the designer's edit cycle: modify JSON → recompile → run tests → verify. This reinforces the "designers don't touch Rust" goal.

4. **Card ID stability caveat**: Document that card IDs are positional (index in the cards Vec). Reordering cards in JSON changes IDs, which breaks saved games and action logs. This is an important architectural constraint that should be visible in the vision.

## roadmap.md suggestions

1. **Mark Step 14 as complete**: Update the status to reflect that configuration externalization is implemented, tested (131 tests, 83% coverage), and documented.

2. **Add a note about card ordering sensitivity** to Step 14's notes: The single-ordered `cards` array design was critical — a naive three-list approach (effects → player cards → encounters) breaks card ID assignment. Future refactoring should preserve this.

3. **Consider a new roadmap step for config validation tooling**: The current system trusts JSON configs to be correct. A validation tool that checks effect name references, card count consistency, and encounter parameter ranges would prevent subtle bugs from config changes. This would pair well with the balancing track.

4. **Consider a roadmap step for runtime config reloading** (dev mode only): Currently configs are baked in at compile time. A dev-mode hot-reload feature would speed up the designer iteration cycle significantly. This could be gated behind a `--dev` flag.

5. **Step 15 (UX polish) should reference the new config architecture**: The designer tooling mentioned in Step 15 could include a config file generator/editor, since configs are now the primary interface for game design.

6. **Balancing track note**: B4 ("LLM-suggested balance changes applied to config files") is now directly enabled by this step. The roadmap could add a cross-reference noting that B4's prerequisite (Step 14) is complete.
