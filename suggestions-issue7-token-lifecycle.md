# Suggestions for vision.md and roadmap.md after Issue 7

## Context
Issue 7 removed `with_default_lifecycle` and `default_lifecycle` methods from `TokenType`, replacing them with explicit `Token::persistent()` and `Token::dodge()` constructors. Card effects now carry an explicit `lifecycle` field.

## Suggestions for vision.md

1. **Update token lifecycle description**: The vision.md should reflect that token lifecycles are now explicit per-effect rather than derived from token type defaults. Dodge is the only token with a non-PersistentCounter lifecycle (`FixedTypeDuration { duration: 1, phases: [Defence] }`).

2. **Add Defence encounter phase**: A new `EncounterPhase::Defence` variant was added to support Dodge's lifecycle. If the vision describes encounter phases, it should include this new phase and clarify when it triggers relative to combat.

3. **CardEffect now carries lifecycle**: The vision should note that `CardEffect` structs explicitly declare the `TokenLifecycle` of the token they grant/modify, making card definitions self-describing.

## Suggestions for roadmap.md

1. **Mark Issue 7 as complete**: "Remove with_default_lifecycle and hardcode token lifecycles" is done.

2. **Follow-up item**: Consider adding a roadmap item to implement actual lifecycle expiration logic — the `FixedTypeDuration` lifecycle on Dodge is stored as metadata but the tick/expiration mechanism is not yet implemented.

3. **Follow-up item**: Consider whether `EncounterPhase::Defence` should trigger as part of combat phase transitions (e.g., when combat enters the Defending phase), and add a roadmap item if so.
