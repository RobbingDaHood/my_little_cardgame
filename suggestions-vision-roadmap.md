# Suggested Improvements to vision.md and roadmap.md

Based on the work done fixing issues.md and completing steps 1–7 leftovers.

## vision.md suggestions

1. **CardDef should declare effects**: The vision mentions cards have types but doesn't specify that card definitions carry declarative effects (token operations with target and amount). Add a note that `CardDef` includes a list of `CardEffect` entries specifying target (self/opponent), token_id, and amount. This is now implemented.

2. **CombatAction is a card play, not a token operation**: The vision should clarify that combat actions are card plays (combatant_id + card_id). Token manipulation (GrantToken, ConsumeToken) happens internally as a result of resolving a card's declared effects — not as separate player-facing actions.

3. **ScoutingParameters replaced by tokens**: The vision's scouting section mentions "preview count, affix bias, pool modifier" as parameters. These are now derived from Foresight tokens rather than stored as an explicit struct. Update the scouting description to say: "Scouting preview count = 1 + Foresight token count. Additional scouting parameters (affix bias, pool modifier) may be derived from other tokens in future steps."

4. **EncounterPhase naming**: The vision doesn't mention specific phase names, but if it does in future, use `NoEncounter`, `Ready`, `InCombat`, `Scouting` (not `Finished` or `PostEncounter`).

5. **HP as tokens**: The vision mentions HP but should clarify that HP is modeled as tokens (`health` and `max_health` in `active_tokens`) rather than as dedicated fields. This aligns with "everything is a token."

## roadmap.md suggestions

1. **Step 6 — CombatAction is now a struct**: The roadmap says "Define CombatState, CombatAction" — update to note that CombatAction is a simple struct `{ combatant_id, card_id }` (not an enum). The old DealDamage/GrantToken/ConsumeToken variants were removed; effects are resolved from CardDef.

2. **Step 6 — No CombatLog type**: The roadmap mentions "deterministic combat log" — clarify that there is no separate CombatLog type. `simulate_combat` returns `CombatState` directly. Combat events are recorded in the canonical ActionLog.

3. **Step 7 — EncounterState is minimal**: The roadmap should note that `EncounterState` contains only `phase: EncounterPhase`. Fields like `encounter_id`, `area_id`, `combat_state`, and `scouting_parameters` were removed as they are derivable from Library state and tokens.

4. **Step 7 — EncounterFinish is system-driven**: The roadmap mentions "pick, fight, replace, scouting" — add a note that finishing an encounter is a system-driven transition (FinishEncounter in EncounterAction), not a player action. Players interact via PickEncounter, PlayCard, and ApplyScouting.

5. **Step 7 — Startup area deck initialization**: The server now initializes with a starter area deck containing 3 combat encounters. The roadmap could note this as part of step 7 acceptance: "Server starts with a playable area deck so developers can immediately test the encounter loop."

6. **Step 7 — /combat/simulate moved to /tests/***: The simulate endpoint is now at `POST /tests/combat/simulate` (temporary testing endpoint). The roadmap should note that test-only endpoints live under `/tests/*`.

7. **Leftover: test endpoint migration**: Tests still use `/tests/*` endpoints for CRUD setup (cards, decks, combat init). A future step should migrate these to use POST /action exclusively, which requires adding card/deck creation actions to the action endpoint.
