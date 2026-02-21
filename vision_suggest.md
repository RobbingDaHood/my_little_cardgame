# Vision Clarification Suggestions

Based on the implementation work completed in the post-step-5 fixes, the following clarifications and amendments to vision.md and roadmap.md are suggested:

## 1. Deck Management API Constraints

**Issue**: The vision and roadmap discuss "Everything is a deck" but don't explicitly clarify that players have exactly 3 fixed decks (Attack, Defence, Resource) that should not be created/deleted via API.

**Suggestion**: Add to vision.md under "Deck types (examples)" section:
- The Player's core decks (Attack, Defence, Resource) are fixed and initialized at game start. Only the Library manages the canonical card definitions and the internal deck representation. The API does not expose deck-creation or deck-deletion endpoints; player deck composition is managed only through adding Library cards to decks via the deck-management flow.

## 2. Single Mutator Endpoint Enforcement

**Issue**: The current implementation has several POST endpoints that are not routed through /action (e.g., POST /tests/combat, POST /tests/cards). While these are testing endpoints, the vision should clarify the boundary.

**Suggestion**: Add to roadmap.md under "Alignment requirements":
- All gameplay state mutations must be performed via POST /action. Testing and debugging endpoints under /tests/* are exceptions and should be documented as temporary testing utilities.

## 3. Area Deck as Single Current Location

**Issue**: The vision mentions "each explorable area is represented by a deck of encounters" but doesn't clarify that at any given time, the player has ONE current area deck representing their current location (not multiple areas loaded simultaneously).

**Suggestion**: Add to vision.md under "Areas as decks (future vision)":
- At any given time, the player has one active area deck representing their current location. Moving to a new area replaces the current area deck. This simplifies the state model and ensures area-specific effects (like scouting bias) are applied to the player's active context.

## 4. Library Naming and Descriptions

**Issue**: The vision states "Intentional minimal canonical data: the authoritative game state deliberately omits titles, verbose descriptions, and flavor text" but the vision itself uses descriptive names like "Iron Ore", "Ore Encounter", "SmallIronVein" which contradict this principle.

**Suggestion**: Clarify in vision.md:
- The canonical Library stores only structural identifiers (card IDs, types, tokens, numeric parameters). All user-facing names, descriptions, and flavor text are delegated to client presentation layers. The API responses use only ID-based references; naming/presentation is the client's responsibility based on a separate design specification.

## 5. Tokens and Token Registry

**Issue**: The vision describes many token types but doesn't clarify which tokens are truly authoritative vs. examples or future-scope.

**Suggestion**: Add explicit sections to roadmap:
- Current token registry (scope of Step 4): Health, Dodge, Stamina (basic survival tokens used in current combat).
- Future token registry (Step 4 onwards): Insight, Renown, Refinement, Stability, Foresight, Momentum, Corruption, Purity, and discipline-specific tokens.
- Each token type must declare its lifecycle (Permanent, PersistentCounter, FixedDuration, etc.) in the canonical registry.

## 6. Scouting Parameters Clarification

**Issue**: ScoutingParams is mentioned in the vision under scouting mechanics but is currently a testing utility not exposed as an API.

**Suggestion**: Clarify in roadmap Step 7 / Step 11 (scouting):
- Scouting parameters (preview count, affix bias, pool modifier) are internal mechanics that influence encounter-generation deterministically during the scouting post-encounter step. They are not user-facing API endpoints but are controlled by the player's scouting action choices and token expenditures (Foresight, etc.).

## 7. API Stability and Endpoint Organization

**Issue**: The vision doesn't discuss the organization of public vs. testing vs. deprecated endpoints.

**Suggestion**: Add a new section to the roadmap introduction:
- **Endpoint Organization**: Public endpoints (without /tests prefix) represent the stable gameplay API. Endpoints under /tests/* are temporary testing utilities and may be removed as implementation progresses. No public gameplay endpoints should be removed during Step 5 post-processing; instead, problematic endpoints are migrated to /tests/*.

## 8. Actions Log as Audit Trail

**Issue**: The vision mentions ActionLog for reproducibility but doesn't emphasize its role in verifying card movement and token lifecycle.

**Suggestion**: Enhance roadmap Step 3 description:
- The append-only ActionLog is the authoritative audit trail for all state changes. Every card movement between zones (Hand, Deck, Discard, Deleted), every token grant/consume/expire, and every random draw are recorded with metadata (reason, amount, timestamp, resulting state) so the game state can be reconstructed from seed + action sequence for validation, testing, and replay.

---

These suggestions aim to clarify boundaries that became apparent during implementation without fundamentally changing the vision or roadmap direction.
