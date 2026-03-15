# Vision & Roadmap Update Suggestions

Based on findings from game_rules.json externalization, hand limit fix, and /version endpoint implementation.

---

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

---

## Additional suggestions from game_rules.json, hand limits, and /version work

### vision.md additions

7. **Add "Configuration Transparency" as a core design principle**: All ~50 game-mechanics constants (death resets, crafting costs, combat rewards, scouting probabilities, milestone scaling, woodcutting patterns) are now externalized to `game_rules.json`. The `/version` endpoint provides a SHA-256 fingerprint of all configs combined with a semver game version, enabling players and tools to verify exactly which rules are in effect.

8. **Document the MaxHand=5 invariant**: The vision describes deck types and hand states but does not codify the 5-card hand limit per kind as a design rule. Each card kind starts with at most 5 cards in the player's hand; excess cards begin in the deck. The MaxHand token per kind enforces this ceiling.

9. **Reference game_rules.json in the config architecture**: The vision mentions `tokens.json` and discipline `cards.json` files but should also mention `game_rules.json` which centralizes constants from 8+ Rust source files.

### roadmap.md additions

7. **Expand Step 14 completion notes**: Beyond card/encounter externalization, ~50 mechanics constants are now in `game_rules.json` covering: death recovery (Health/Stamina 1000), combat milestone rewards (100 Insight), research cost scaling (base 10, multiplier 2), crafting cost formulas (divisor 4, material cap 75%, counts 2-4), scouting mutation probabilities, milestone insight costs, and all 16 woodcutting pattern evaluations.

8. **Add /version endpoint to Step 15 or nearby**: `GET /version` returns `{"version": "0.0.1-<hash>", "game_version": "0.0.1", "config_hash": "<8-char-sha256>"}`. The hash covers all 12 embedded config JSONs in deterministic order. This directly enables the save-game replay story at roadmap line 633-635.

9. **Update B2 (General Config Bypass) scope**: With `game_rules.json` now containing all mechanics constants, B2 reviewers should audit those values too (death_reset=1000, milestone_insight=100, crafting formulas, scouting probabilities). The `/version` endpoint makes it trivial to verify config changes took effect.

10. **Note hand limit normalization as a balance change**: All discipline initial hand counts were reduced to exactly 5 per kind (was 7-20). This is a BREAKING change for saved action logs. The reduced starting hand increases importance of draw mechanics and early-game decisions.

11. **Consider config schema versioning**: Now that `game_rules.json` exists and `/version` fingerprints configs, add a future step for migration when schema changes. Embed a `"schema_version"` field in game_rules.json and validate at load time. Critical when save-game replay needs historical configs.
