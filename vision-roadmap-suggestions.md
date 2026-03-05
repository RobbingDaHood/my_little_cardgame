# Vision & Roadmap Improvement Suggestions

Discovered during implementation of mining redesign (9.5) and issues.md fixes.

## Vision.md Suggestions

1. **Document the token-based card effect pattern as a design principle.** Mining now uses `costs`/`gains` vectors of `GatheringCost` with token types determining behavior at resolution time. This "interpret token types at resolution" pattern is powerful and could become the standard approach for all future discipline card effects. Vision.md should explicitly call this out as a recommended pattern: "Card effects should express all behavior through token types in cost/gain vectors rather than custom-named fields. Resolution logic interprets the token type to determine behavior."

2. **Add EncounterConcludeEncounter to the encounter lifecycle description.** The vision describes encounter lifecycle phases but doesn't mention that some encounters support player-initiated conclusion (distinct from abort). Suggest adding a note that encounters can have three endings: win-by-objective (combat), conclude-by-choice (mining), and loss (all types). This distinction affects how future encounter types should be designed.

3. **Clarify encounter-scoped vs persistent tokens.** Vision.md mentions encounter-scoped tokens but the current implementation uses two different mechanisms: some tokens live on encounter state structs (RestToken on RestEncounterState, enemy tokens on CombatEncounterState) while others live on player `token_balances` and are manually reset (MiningLightLevel, MiningYield, MiningPower, FishingRangeMin/Max, FishAmount). Consider standardizing on one approach or explicitly documenting the tradeoffs of each.

4. **OreHealth token type is now unused.** Mining no longer uses OreHealth (kept for backward compatibility). Vision.md still lists it in the token type enum. Consider marking it as deprecated or removing it in a future cleanup pass. Same applies to the `ore_tokens` mention in encounter state descriptions.

5. **Hardcoded card IDs in scenario tests are a recurring issue.** Multiple tests were broken by card ID changes. Vision.md could add a testing guideline: "Scenario tests must discover card IDs dynamically via API queries (e.g., `/library/cards?card_kind=Mining&location=Hand`) rather than hardcoding index-based IDs, since card registration order determines IDs and any card addition/removal shifts all subsequent IDs."

## Roadmap.md Suggestions

1. **Add a roadmap item for standardizing all gathering disciplines to the token-based pattern.** Mining's redesign uses `costs`/`gains` vectors with `GatheringCost` for everything. Herbalism, Woodcutting, and Fishing still use discipline-specific fields (e.g., `herb_power`, `match_color`, `chop_pattern`, `fish_range_shift`). A future roadmap item could propose migrating each to the same token-based approach, making disciplines more consistent and easier to extend.

2. **Add a roadmap item for fixing remaining hardcoded card IDs in scenario tests.** 11 scenario tests still fail on main due to hardcoded card IDs. This should be tracked as a code quality item. The pattern established in the mining tests (dynamic discovery via API) should be applied to all remaining tests.

3. **Consider adding a `ConcludeEncounter` action to other encounter types.** The mining redesign introduced `EncounterConcludeEncounter` as a generic action name. This could be useful for fishing (stop fishing early to keep current catch), woodcutting (stop chopping to keep lumber), etc. Each would calculate rewards differently but share the same action dispatch pattern. This could be a roadmap item under "encounter enhancements."

4. **Rest token progression is now easier to implement.** With `RestDef { rest_token_min, rest_token_max }` on each encounter card, rest progression can be achieved simply by adding higher-tier rest encounter cards with larger token ranges (e.g., 3-4 or 5-6 rest tokens). These could be unlocked via milestones or research. The roadmap's "Rest token progression" gap item should be updated to reference this mechanism.

5. **Mining card balance needs playtesting.** The current card values (power 300-1000, light 200-350, ore damage 50-200, light_level_cap 500-600) are initial estimates. Add a roadmap item for a playtesting/tuning pass after the core mechanics are stable. The yield formula (`power × light / 100`) means that with light=300 and power=500, one card play yields 1500 — which is substantial relative to Stamina: 1000. This may need rebalancing.

6. **Consider removing the `test_player_kills_enemy_and_combat_ends` test isolation issue.** This test in flow_tests.rs fails when run alongside other tests due to shared Rocket state but passes alone. It's pre-existing and unrelated to mining but should be tracked. It could be fixed by creating a fresh game state per test instance.
