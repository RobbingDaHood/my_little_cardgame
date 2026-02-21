# Issues and suggested fixes (cleanup: after step 7, before step 8)

This document collects issues observed while implementing the encounter loop (post-step 7) and proposes minimal fixes to align the implementation with vision.md and roadmap.md (pre-step 8).

## EncounterPick — use card_id, not area_id

EncounterPick should accept a card_id rather than an area_id because the encounter is a card drawn from the area deck's hand (a Library card).

Requirements:
- The card must be an area-type card.
- At least one copy must be "on hand".

When both requirements are satisfied, the card can be played.

## EncounterPlayCard — avoid deck_id

Do not require explicit deck_id values. Every card type belongs to an implicit deck of that type; callers should provide a card_id and the server should validate deck semantics.

When playing a card (by card_id) ensure:
- The card has the correct type.
- There is at least one copy on hand.

Handle these semantics via the Library/implicit-deck model; avoid exposing explicit deck_id fields in the public API.

## EncounterApplyScouting — accept card_id(s)

EncounterApplyScouting should use card_id with the same restrictions described above. It may accept a list of card_id values when the action involves multiple cards.

## EncounterFinish should not be a player action

After an encounter starts, players play cards until it finishes; the system then applies the scouting update (EncounterApplyScouting) and presents the next encounter. There is no need for a separate player-invoked EncounterFinish action.

## /combat/simulate should not be public

Combat resolution must be driven through the single mutator action endpoint (POST /action). If /combat/simulate is useful for debugging or tests, move it under the test endpoints (e.g., /tests/*) and document it as temporary.

## CombatEvent is rarely needed

The ActionLog records player actions. Combined with the initial seed, the action log is sufficient to reproduce state; avoid adding redundant reporting events like CombatEvent unless they provide metadata that cannot be reconstructed from seed+action log.

## Combatant and HP modelling

Combatants do not need separate external IDs for players. current_hp and max_hp should be modelled as tokens rather than ad-hoc fields — consistent with the "everything is a deck or a token" principle.

## CombatAction and Combat state

CombatAction is largely redundant if Encounter actions already capture the available plays. Combat state should be derivable from:
- the player's tokens,
- the combatant's tokens,
- card states stored in the Library (counts/locations).

Keep the runtime state minimal and rely on the ActionLog + seed for replayability.

## CombatLogEntry / CombatLog

There should be a single canonical log: the ActionLog. Avoid introducing a separate CombatLog unless there is a strong, documented need for a secondary reporting trail.

## EncounterState and EncounterPhase

EncounterState does not need separate fields for encounter_id, area_id, combat_state, or scouting_parameters; these can be reconstructed from tokens and Library state. One EncounterPhase enum is sufficient; refine phase names rather than adding many ad-hoc flags.

Suggested EncounterPhase adjustments:
- Remove a separate "ready" flag.
- Expand "InCombat" to include explicit combat sub-phases (resolve tick, resolve turn, etc.) if needed.
- Replace "Finished" with a neutral state like "NoEncounter".
- Rename "PostEncounter" to "Scouting" for clarity.

## ScoutingParameters

Represent scouting-related parameters via tokens and deterministic parameters derived from those tokens. Scouting should deterministically bias replacement generation using token-driven parameters and the ActionLog; do not hard-code separate global bias structures.

## Combat module placement

Combat logic should interact with the Library and the canonical data model but does not need to live in library.rs. Place combat code in an appropriate core module while preserving the Library as the authoritative card registry.

## resolve_combat_tick signature

Prefer a card_id-based API (resolve_combat_tick(card_id, ...)) that:
1. Reads and validates the card definition and type.
2. Checks that the card's cost can be paid and returns a clear error if not.
3. Applies the card's effect by manipulating tokens (player or combatant tokens).
4. Checks victory/defeat conditions (e.g., HP tokens reaching zero) and transitions to the Scouting phase when resolved.

## encounter.apply_action

Update encounter.apply_action to follow the rules above and to rely on card_id-based actions, token manipulation, and the single POST /action mutator.

## Startup: initialize area decks for quick play

Ensure the server initialization populates the Library with a small set of combat encounters (correct counts) and marks some copies "on hand" so a developer can immediately pick an encounter and progress through the simple combat loop after startup.

## Simple combat is fine initially

The first combat implementation can be deliberately minimal (attack cards that reduce HP). Add dodge, stamina, and other complexity incrementally in later roadmap steps.

## Suggested clarifications to vision.md and roadmap.md

Suggest changes to vision.md and roadmap.md to avoid making the same mistake again. These suggestions should be put in a file, the file may NOT be placed in docs/design.


---

This file is a cleanup note following step 7 (encounter play loop) and intended to be applied before proceeding with step 8 (expanding encounter variety).
