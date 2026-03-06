# Vision and Roadmap Improvement Suggestions

Generated during the post-9.5 "Better Mining redesign" cleanup.

## Vision.md suggestions

1. **Consolidate duplicate "Current combat setup" sections**: There are two `## Current combat setup` headings (around lines 88 and 124). Merge these into a single coherent section to avoid confusion.

2. **Document encounter-scoped token storage pattern**: Now that encounter-scoped tokens live on EncounterState structs (not in global token_balances), vision.md should explicitly describe this pattern as a design principle — encounter state owns encounter-scoped data; global token_balances only holds persistent player tokens.

3. **Add a "Player death and recovery" section**: The death mechanic (gathering material reset, health/stamina restoration, PlayerDeaths counter) is now implemented but only briefly mentioned inline. It deserves its own subsection under gameplay elements.

4. **Update mining encounter template (line ~613-620)**: The mining template still references the old OreHealth-based mining model. The current model uses a yield-accumulation approach where ore is granted on conclude based on `min(stamina, yield)`. This section should be fully rewritten to match the current implementation.

5. **Clarify TokenAmount as the universal token quantity struct**: TokenAmount is now used in costs, gains, and damages across all gathering disciplines. Vision.md should emphasize this as a core design pattern — all token-quantity expressions use the same struct with optional per-gain caps.

6. **Document EncounterConcludeEncounter as a pattern**: All gathering disciplines now support voluntary conclusion. This should be documented as a standard encounter flow pattern: play cards → accumulate rewards → conclude (win) or abort (loss).

## Roadmap.md suggestions

1. **Add a "9.5 post-cleanup" summary**: The 9.5 section should note that a cleanup pass was done (token restructuring, encounter-scoped token migration, player death mechanic, dynamic test IDs, conclude for all disciplines).

2. **Step 10 (Research) prerequisite note**: Research encounters reference Insight tokens — consider noting that the Insight card effect infrastructure is already partially in place (MilestoneInsight token type exists, Insight token type exists).

3. **Step 12 (edge cases)**: Player death is now implemented, so "player death and recovery" can be checked off or noted as partially complete in this step's description.

4. **Step 13 (Milestones)**: Consider adding that PlayerDeaths token could factor into milestone difficulty scaling — dying more makes future milestones harder.

5. **Step 14 (Configuration)**: Consider noting that the current card registration pattern (hardcoded in Rust discipline modules) is the specific thing being externalized — this gives a clearer picture of the migration scope.

6. **Consider adding a "Technical debt" or "Cleanup" tracking section**: This cleanup uncovered that hardcoded card IDs in tests, outdated type names, and undocumented encounter patterns can accumulate. A lightweight section tracking known cleanup items would help future contributors.
