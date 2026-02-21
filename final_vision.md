# Final Vision: My Little Cardgame (Post-Step-5 Refinements)

This is an updated version of the vision.md incorporating clarifications from the post-step-5 implementation work. Changes are minimal and focus on clarifying boundaries that became apparent during implementation.

## Key Clarifications

### Player Decks Are Fixed

The Player's core decks (Attack, Defence, Resource) are fixed and initialized at game start. Only the Library manages the canonical card definitions and the internal deck representation. The API does not expose deck-creation or deck-deletion endpoints; player deck composition is managed only through adding Library cards to decks via the deck-management flow.

When the vision refers to "Everything is a deck", Player decks specifically mean the three fixed Card/Token/Resource decks whose membership is managed by the player via Library integration—not arbitrary user-created decks.

### Single Mutator Endpoint Rule

All gameplay state mutations must be performed via POST /action. Testing and debugging endpoints under /tests/* are exceptions and should be documented as temporary testing utilities. As features graduate from testing to stable, their endpoints should be migrated to /action-backed flows.

### Area Deck Represents Current Location

At any given time, the player has one active area deck representing their current location (not multiple areas loaded simultaneously). Moving to a new area replaces the current area deck. This simplifies the state model and ensures area-specific effects (like scouting bias) apply to the player's active context.

### Canonical Data Omits Flavor

The canonical Library stores only structural identifiers (card IDs, types, tokens, numeric parameters). All user-facing names, descriptions, and flavor text are delegated to client presentation layers. The API responses use only ID-based references; naming/presentation is the client's responsibility based on a separate design specification. This ensures the game state remains minimal, reproducible, and suitable for replay and analysis.

### Token Registry Scope

The canonical token list grows incrementally with the roadmap:
- **Current scope (Step 1-5)**: Health, Dodge, Stamina (basic survival tokens used in current combat).
- **Future scope (Step 4 onwards)**: Insight, Renown, Refinement, Stability, Foresight, Momentum, Corruption, Purity, and discipline-specific tokens.

Each token type must declare its lifecycle (Permanent, PersistentCounter, FixedDuration, etc.) in the canonical registry.

### Scouting Parameters Are Internal

Scouting parameters (preview count, affix bias, pool modifier) are internal mechanics that influence encounter-generation deterministically during the scouting post-encounter step. They are not user-facing API endpoints but are controlled by the player's scouting action choices and token expenditures (Foresight, etc.).

### Actions Log Is the Audit Trail

The append-only ActionLog is the authoritative audit trail for all state changes. Every card movement between zones (Hand, Deck, Discard, Deleted), every token grant/consume/expire, and every random draw are recorded with metadata (reason, amount, timestamp, resulting state) so the game state can be reconstructed from seed + action sequence for validation, testing, and replay.

---

All other content from the original vision.md remains as written. These clarifications ensure the implementation aligns with the stated vision while providing additional clarity on boundaries that were implicit in the original document.
