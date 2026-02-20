# My Little Card Game

A card game where **everything is a deck!** This is a web-based card game API built with Rust and Rocket, featuring a unique mechanic where all game elements are represented as decks of cards.

## Game Concept

The core mechanic revolves around three types of decks:
- **Attack Deck**: Contains cards used for offensive actions
- **Defence Deck**: Contains cards used for defensive actions  
- **Resource Deck**: Contains cards that provide resources and effects

Cards can be in different states (Deck, Hand, Discarded, Deleted) and moved between these states during gameplay. Combat is resolved by playing cards from your hand with various effects and costs.

## Features

- RESTful API for card and deck management
- Combat system with card state transitions
- OpenAPI/Swagger documentation
- Comprehensive test coverage (13 integration tests)
- Input validation and descriptive error messages

## Prerequisites

- Rust 1.93.0 or later
- Cargo (comes with Rust)

## Installation

1. Clone the repository:
```bash
git clone https://github.com/RobbingDaHood/my_little_cardgame.git
cd my_little_cardgame
```

2. Build the project:
```bash
cargo build --release
```

## Running the Server

Start the development server:
```bash
cargo run
```

The server will start on `http://localhost:8000` by default.

## API Documentation

Once the server is running, access the interactive Swagger UI documentation at:
```
http://localhost:8000/swagger/
```

### Key Endpoints

#### Cards
- `GET /cards` - List all cards
- `GET /cards/<id>` - Get a specific card
- `POST /cards` - Create a new card

#### Decks
- `GET /decks` - List all decks
- `GET /decks/<id>` - Get a specific deck
- `POST /decks` - Create a new deck
- `POST /decks/<id>/cards` - Add a card to a deck
- `GET /decks/<deck_id>/cards/<card_id>` - Get a specific card in a deck
- `DELETE /decks/<deck_id>/cards/<card_id>` - Remove a card from a deck

#### Combat
- `GET /combat` - Get current combat state
- `POST /combat` - Initialize a new combat encounter
- `POST /play` - Play a card action during combat

### Example Requests

Create a new attack card:
```bash
curl -X POST http://localhost:8000/cards \
  -H "Content-Type: application/json" \
  -d '{
    "card_type_id": 1,
    "card_type": "Attack",
    "effects": [],
    "costs": [],
    "count": 10
  }'
```

Create a new deck:
```bash
curl -X POST http://localhost:8000/decks \
  -H "Content-Type: application/json" \
  -d '{
    "contains_card_types": ["Attack", "Defence"]
  }'
```

Initialize combat:
```bash
curl -X POST http://localhost:8000/combat
```

## Development

### Seeding and reproducibility

- Set the session RNG seed: POST /player/seed with JSON body `{ "seed": 42 }` to initialize deterministic runs.
- The server records the seed (and key RNG draws) in the ActionLog (GET /actions/log) so runs can be reproduced from seed + action log.
- Snapshot and restore helpers are available in the player_seed utilities to serialize RNG state (used for replay or debugging).



### Running Tests

Run the full test suite:
```bash
cargo test
```

Run tests with output:
```bash
cargo test -- --nocapture
```

### Code Quality

Check for linting issues:
```bash
cargo clippy --all-targets --all-features -- -D warnings
```
Run this command before each commit to ensure zero clippy warnings.

Format code:
```bash
cargo fmt
```

Ensure formatting checks pass before committing:
```bash
cargo fmt -- --check
```

### Project Structure

```
src/
├── lib.rs              # Library entry point with public API
├── main.rs             # Binary entry point
├── action/             # Player action handling
├── combat/             # Combat system logic
├── deck/               # Deck and card management
│   ├── mod.rs         # Deck operations
│   ├── card.rs        # Card types and operations
│   └── token.rs       # Effect and cost tokens
├── player_data.rs      # Player state management
└── status_messages.rs  # API response messages

tests/
└── api_tests.rs        # Integration tests via HTTP API
```

## Card States

Cards transition through different states during gameplay:
- **Deck**: Card is in the deck ready to be drawn
- **Hand**: Card has been drawn and is in the player's hand
- **Discarded**: Card has been played and is in the discard pile
- **Deleted**: Card has been removed from the game

## Design Philosophy

- **Encapsulation**: Internal APIs remain private; all interactions go through public HTTP endpoints
- **Type Safety**: Leverages Rust's type system for correctness
- **Error Handling**: No panics in production code; all errors return proper HTTP status codes
- **Testing**: Comprehensive integration tests covering all endpoints and edge cases

## Contributing

This project follows modern Rust best practices:
- Zero clippy warnings
- No unwrap() calls in production code
- Comprehensive error handling
- Meaningful commit messages

## License

Apache-2.0

## Author

RobbingDaHood

### Developer setup - pre-commit hooks

Install pre-commit (pip install --user pre-commit) and enable the hooks:

```bash
make install-hooks
# or
./scripts/install-hooks.sh
```

The hooks run:
- `cargo fmt -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `make coverage` (enforces 85% coverage)


## Development notes

See docs/dev/CONTRIBUTING.md and docs/dev/SECURITY.md for developer guidelines, testing, and security audit instructions.

## Documentation Structure

- `docs/design/` — Vision, roadmap, and current state
  - `vision.md` — High-level design principles and core mechanics
  - `roadmap.md` — Implementation roadmap with 18 steps
  - `current_state.md` — Current implementation status
- `docs/dev/` — Developer guidance
  - `CONTRIBUTING.md` — Code standards and testing expectations
  - `SECURITY.md` — Security audit procedures
- `docs/audits/` — Security and quality audits
  - `AUDIT_FINDINGS.md` — Key findings from code audit
  - `AUDIT_ISSUES.md` — Unwrap/panic/expect usage tracking
- `docs/issues/` — Specific technical issues and fixes
- `docs/examples/` — Example API usage and scripts

Run tests: 
running 4 tests
test deck::tests::change_card_state_missing_state_returns_error ... ok
test deck::tests::change_card_state_nonexistent_card_returns_error ... ok
test deck::tests::change_random_cards_state_calls_change_card_state_error_when_old_state_missing ... ok
test deck::tests::draw_cards_moves_to_hand ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 2 tests
test grant_token_via_action ... ok
test set_seed_via_action_records_log ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s


running 1 test
test hello_world ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s


running 9 tests
test test_add_card_to_nonexistent_deck ... ok
test test_add_duplicate_card_to_deck ... ok
test test_add_wrong_card_type_to_deck ... ok
test test_create_card_with_zero_count_rejected ... ok
test test_create_deck_with_empty_card_types_rejected ... ok
test test_get_combat_before_initialization ... ok
test test_get_nonexistent_card ... ok
test test_play_card_without_active_combat ... ok
test test_play_nonexistent_card_in_combat ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s


running 1 test
test action_log_stress_append ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s


running 8 tests
test test_dodge_consumed_by_enemy_attack ... ok
test test_advance_phase_rotates_state ... ok
test test_enemy_play_adds_dodge_to_enemy_in_defending ... ok
test test_enemy_unit_hand_to_discard_on_play ... ok
test test_phase_enforcement_attack_in_defending_should_fail ... ok
test test_player_kills_enemy_and_combat_ends ... ok
test test_play_defence_moves_card_to_discard ... ok
test test_seed_determinism_for_enemy_selection ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s


running 1 test
test library_tokens_endpoint_returns_list ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s


running 2 tests
test grant_and_replay ... ok
test action_log_concurrent_append ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test proptest_replay_preserves_balances ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s


running 1 test
test replay_from_log_reproduces_balances_randomized ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 2 tests
test test_play_defence_card_adds_tokens ... ok
test test_play_defence_card_health_kills_enemy ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s


running 1 test
test src/lib.rs - rocket_initialize (line 46) - compile ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
Run coverage locally: Filename                      Regions    Missed Regions     Cover   Functions  Missed Functions  Executed       Lines      Missed Lines     Cover    Branches   Missed Branches     Cover
-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
action/mod.rs                     103                 7    93.20