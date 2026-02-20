# Step 5 Implementation Summary: Area Decks with Encounter Removal + Replacement

## Overview
This document summarizes the implementation of Step 5 from the roadmap: **Add Area Decks with encounter removal + replacement and scouting hooks**.

## What Was Implemented

### 1. **AreaDeck Core Types** (`src/area_deck/mod.rs`)
- `AreaDeck` struct managing a collection of encounters
- `Encounter` struct with lifecycle states (Available, Active, Resolved)
- `AffixPipeline` for deterministic affix generation based on seeds
- Encounter properties: id, name, base_type, state, affixes, entry_cost, reward_deck_id

### 2. **Scouting System** (`src/area_deck/scouting.rs`)
- `ScoutingParams` struct controlling replacement behavior
- Deterministic seed modification based on scouting parameters
- Support for affix biases and pool modifiers
- All scouting decisions influence encounter generation deterministically

### 3. **HTTP Endpoints** (Read-Only)
- `GET /area/{id}` - Retrieve area deck details
- `GET /area/{id}/encounters` - List all encounters in an area
- Both endpoints return full state for inspection

### 4. **Player Actions via `/action` Endpoint** (Single Mutation Point)
- `PlayerActions::DrawEncounter` - Activate an encounter
- `PlayerActions::ReplaceEncounter` - Replace resolved encounter with new one
- `PlayerActions::ApplyScouting` - Apply scouting parameters for next replacement
- All mutations go through single `/action` endpoint (architecture requirement)

### 5. **ActionLog Integration**
- `ActionPayload::DrawEncounter` - Records when encounters are drawn
- `ActionPayload::ReplaceEncounter` - Records replacement with affixes applied
- `ActionPayload::ApplyScouting` - Records scouting decisions
- Enables full auditability and replay for deterministic runs

### 6. **Deterministic Replacement Generation**
- Same seed + same scouting params = same encounter affixes
- AffixPipeline uses Lcg64Xsh32 RNG seeded from input
- ScoutingParams.apply_to_seed() deterministically modifies base seed
- Verified with multiple test cases

## Architecture Decisions

### Single Mutation Endpoint
All state mutations route through `POST /action` with `PlayerActions` enum variants:
- Maintains central audit trail via ActionLog
- Simplifies transaction semantics
- Enables deterministic replay from seed + action sequence
- Read-only operations via separate GET endpoints (no mutation)

### Encounter Lifecycle
```
Available -> Active -> Resolved -> Replaced
```
- **Available**: Ready to be drawn from the area deck
- **Active**: Currently being interacted with (e.g., combat in progress)
- **Resolved**: Completed and ready for replacement
- Resolved encounters can only be replaced (not re-drawn)

### Deterministic Operations
Every random operation (affix selection) uses:
1. Session RNG seed (from player state)
2. Scouting parameters (if applied)
3. Encounter/area specific seed derivations
Result: Identical sequences can be replayed from seed + action log

## Test Coverage

### Unit Tests (4 tests in `src/area_deck/mod.rs`)
- Area deck creation and initialization
- Draw/activate encounter transitions
- Resolve and replace lifecycle
- Deterministic affix generation from same seed

### Integration Tests (11 tests in `tests/area_deck_integration.rs`)
- Draw and replace with state verification
- Scouting parameter influence on seeds
- Entry cost and reward deck binding
- Available encounters filtering
- Replacement of non-available encounters (error case)
- ActionPayload serialization for all variants

### End-to-End Tests (5 tests in `tests/area_deck_e2e.rs`)
- Full workflow: draw → resolve → replace → scouting
- ActionLog recording with GameState integration
- Deterministic replay verification
- Scouting parameter effect on seed derivation
- Encounter ID incrementing across generations

**Total new tests: 20**

## Step 5 Acceptance Criteria ✅

From roadmap.md:
- ✅ Drawing and resolving an area encounter removes it from the area deck
- ✅ Immediately creates a replacement entry (via replace operation)
- ✅ Scouting-related parameters can bias replacement generation in deterministic tests
- ✅ Entry cost and reward deck binding supported
- ✅ All deck-bound draws, replacements recorded in ActionLog
- ✅ Deterministic behavior verified with seeded tests
- ✅ Zero Clippy warnings
- ✅ All tests pass

## Files Structure

```
src/area_deck/
  ├── mod.rs           # Core AreaDeck, Encounter, AffixPipeline types
  ├── endpoints.rs     # GET /area/{id} and /area/{id}/encounters
  └── scouting.rs      # ScoutingParams with deterministic seed modification

tests/
  ├── area_deck_integration.rs  # 11 integration tests
  └── area_deck_e2e.rs          # 5 end-to-end tests
```

## Usage Example

```rust
// Create an area deck
let mut forest = AreaDeck::new("forest_1".to_string(), "Enchanted Forest".to_string());

// Add encounters
let encounter = Encounter::new("enc_1".to_string(), "Goblin".to_string(), "Combat".to_string())
    .with_entry_cost(10)
    .with_reward_deck("rewards_1".to_string());
forest.add_encounter(encounter);

// Draw (activate) an encounter via /action endpoint
// POST /action { "DrawEncounter": { "area_id": "forest_1", "encounter_id": "enc_1" } }

// Resolve it
forest.resolve_encounter("enc_1").unwrap();

// Generate replacement with scouting bias
let params = ScoutingParams::new(2).with_affix_bias(vec!["blessed".to_string()]);
let seed = params.apply_to_seed(42);
let new_encounter = forest.generate_encounter("Combat".to_string(), seed);

// Replace via /action endpoint
// POST /action { "ReplaceEncounter": { "area_id": "forest_1", "old_encounter_id": "enc_1", "new_encounter_id": "enc_2" } }
```

## Integration Points

### With Combat System
- Encounter can have associated combat via combat start action
- Entry cost should be consumed at combat start (future integration)
- Reward deck distributed when encounter is won

### With ActionLog
- All AreaDeck operations recorded with reason/metadata
- Enables full audit trail and replay
- Part of deterministic session replay capability

### With Player State
- AreaDecks stored in PlayerData.area_decks HashMap
- Accessed via concurrent-safe Mutex<HashMap>
- Per-session scouting state would be added in future

## Future Enhancements (Post Step 5)

1. **Entry Cost Consumption** - Deduct entry_cost token at encounter start
2. **Reward Distribution** - Distribute cards from reward_deck on victory
3. **Modifier Pulls** - Support modifier deck pulls for affix selection
4. **Extended Scouting** - Preview count, candidate pool filtering
5. **Encounter Templates** - Seed-driven base type generation
6. **Persistent Areas** - Disk-backed area deck state

## Commits

This implementation was delivered as 6 focused commits, each passing all tests:

1. **Commit 1**: Core AreaDeck types and operations
2. **Commit 2**: Action handlers for mutations (DrawEncounter, ReplaceEncounter, ApplyScouting)
3. **Commit 3**: GET endpoints for read-only access
4. **Commit 4**: Scouting parameters with deterministic seed modification
5. **Commit 5**: Integration tests (11 tests)
6. **Commit 6**: End-to-end tests with ActionLog (5 tests)

All commits maintain zero Clippy warnings and passing test suite.

## Alignment with Vision

This implementation fulfills the key vision principles:

1. **Everything is a Deck** ✅
   - AreaDeck is a deck of encounter cards
   - Encounters are cards that transition through states
   - Rewards distributed via reward decks

2. **Single-Seed Reproducibility** ✅
   - All random operations derive from session seed
   - Scouting parameters deterministically modify seed
   - ActionLog records all decisions for replay

3. **Canonical Library Semantics** ✅
   - Encounter definitions can reference reward decks
   - Future: crafted card copies linked to library

4. **Token Lifecycle & ActionLog** ✅
   - All AreaDeck operations recorded with full metadata
   - Reason, actor, resulting state captured
   - Enables full audit trail

5. **Single Mutator Endpoint** ✅
   - All mutations go through POST /action
   - PlayerActions enum differentiates operations
   - GET endpoints remain read-only
