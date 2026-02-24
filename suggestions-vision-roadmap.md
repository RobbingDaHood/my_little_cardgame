# Suggestions for vision.md and roadmap.md

All suggestions from the post-7.7 implementation have been applied to vision.md and roadmap.md.

---

## Resolved contradictions and clarifications

1. **Enemy resource draws**: Resource card draws apply to the same deck type only. When a resource card triggers DrawCards, it draws cards for **each of the three deck types** (attack, defence, resource), so all deck types have a replenishment mechanism.

2. **CardEffect deck future usage**: The EnemyCardEffect deck will be used during the post-encounter scouting phase to help generate new encounters for the encounter deck. The PlayerCardEffect deck will be used during research to help generate new cards for the library. Detailed mechanics will be fleshed out in later roadmap steps.

3. **Card ID stability**: New cards must always be appended to the end of the Library vector, never inserted at the beginning, to preserve stable card IDs.

4. **DrawCards refactored**: DrawCards has been refactored from a TokenType variant into a proper `CardEffectKind::DrawCards { amount }` variant. Vision.md and roadmap.md have been updated to reflect this.

---

## Suggestions from 7.7-cleanup round

### vision.md

1. **Module structure**: Vision.md still references `src/combat/` for combat resolution but doesn't mention the `src/library/` module directory split. Consider adding a brief architecture note listing the main modules (`src/library/{types, combat, encounter, registry, action_log, game_state, endpoints}`) so vision.md serves as the single source of truth for code organization.

2. **CardEffectKind extensibility**: Vision.md now documents the two CardEffectKind variants (ChangeTokens, DrawCards). Consider documenting the intended extensibility pattern — that new card effect subtypes (e.g., conditional effects, area-of-effect) should be added as new CardEffectKind variants rather than overloading ChangeTokens with special token types.

3. **Combat flow documentation**: The auto-advance behavior (enemy plays + phase advance after each player card) is documented in vision.md but could be clearer. The exact sequence is: player plays card → resolve player card effects → resolve enemy play (random card matching phase) → advance phase → check combat end. Documenting this as a numbered sequence would help implementers.

4. **Health initialization**: Player health is initialized to 20 when picking an encounter only if current health is 0. This implicit initialization is not documented in vision.md. Consider documenting the rule: "Player starts each game with Health set to 20 on first encounter pick; health persists across encounters within a game."

5. **Scouting after loss**: After an enemy win, the game transitions to Scouting phase and the player can still apply scouting. This behavior (scouting after loss) is not explicitly documented. Consider clarifying whether this is intentional or whether losing should skip scouting.

### roadmap.md

6. **Post-7.7 cleanup section**: The roadmap should add a "Post-7.7 cleanup" section documenting the changes made:
   - Removed `EncounterPhase::Defence` (now uses `CombatPhase::Defending`)
   - Removed `Combatant` struct (enemy tokens moved directly to `CombatState.enemy_tokens`)
   - Extracted `DrawCards` from `TokenType` into `CardEffectKind` enum
   - Increased DrawCards amount from 1 to 2
   - Split `library.rs` into `src/library/` module directory
   - Added long-scenario integration tests (`tests/scenario_tests.rs`)

7. **Test migration progress**: The roadmap says "Try to migrate tests away from test endpoints and use only public endpoints." The new scenario tests follow this guidance. Consider tracking which test files still depend on test endpoints vs which use only production endpoints, and setting a target for full migration.

8. **Scenario test expectations in roadmap**: Future roadmap steps (8+) should mention updating `tests/scenario_tests.rs` as part of their acceptance criteria. Each new encounter type or gameplay feature should have at least one scenario test demonstrating the full loop.

### Contradictions found

9. **EncounterPhase enum in vision.md line 22**: Vision.md references `EncounterPhase` variants but doesn't list them. The current variants are: Ready, InCombat, Scouting, NoEncounter. Consider listing them explicitly since they define the encounter state machine.

10. **DrawCards amount inconsistency**: The roadmap step 7.6 says "starting decks for both players and enemies contain approximately 50% draw/resource cards" but doesn't specify the draw amount. Now that DrawCards draws 2 cards, this affects deck pacing significantly — consider documenting the draw amount in the roadmap step or in a "Current balance parameters" section.
