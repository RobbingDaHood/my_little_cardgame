# My Little Card Game

A card game where **everything is a deck!** This is a web-based card game API built with Rust and Rocket, featuring a unique mechanic where all game elements — combat, gathering, crafting, research — are represented as decks of cards across eight disciplines.

## Game Concept

The core mechanic revolves around **encounters** drawn from your hand. Each turn you pick an encounter card (Combat, Mining, Herbalism, Woodcutting, Fishing, Crafting, Research, or Rest), play discipline cards to resolve it, then scout for new encounter cards. Cards cycle through states: **Deck → Hand → Discarded → Deleted**.

Tokens represent persistent resources (Health, Stamina, Insight, materials) that carry between encounters — managing them is the strategic heart of the game.

## Features

- RESTful API with 12+ endpoints for full gameplay
- Eight encounter disciplines with distinct mechanics
- Deterministic replay via seed + action log
- Externalized game-rules configuration (`configurations/general/game_rules.json`)
- Self-documenting API: `/docs/tutorial`, `/docs/hints`, `/docs/designer`
- Version fingerprint via `GET /version` (game version + config hash)
- Session metrics via `GET /metrics`
- OpenAPI/Swagger documentation at `/swagger/`
- Comprehensive test coverage (57+ integration tests, ≥80% line coverage)
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

### Custom Port Configuration

The server supports Rocket's built-in environment variable configuration. To run on a different port:

```bash
ROCKET_PORT=8001 cargo run
```

This enables running multiple instances simultaneously for parallel balancing runs or testing:

```bash
ROCKET_PORT=8001 cargo run &   # Instance 1
ROCKET_PORT=8002 cargo run &   # Instance 2
```

Any `ROCKET_` prefixed environment variable is supported (e.g., `ROCKET_ADDRESS=0.0.0.0` to bind to all interfaces). See [Rocket configuration docs](https://rocket.rs/v0.5/guide/configuration/) for all options.

## API Documentation

Once the server is running, access the interactive Swagger UI documentation at:
```
http://localhost:8000/swagger/
```

The game also provides self-documenting endpoints that explain gameplay without needing to read source code:

| Endpoint | Purpose |
|----------|---------|
| `GET /docs/tutorial` | Step-by-step new-player walkthrough |
| `GET /docs/hints` | Strategies and tips per discipline |
| `GET /docs/designer` | Encounter/card/token authoring reference |

### Key Endpoints

#### Game Actions
- `POST /action` — Submit a player action (NewGame, EncounterPickEncounter, EncounterPlayCard, ResearchPlayHand, ResearchConcludeExperiment, etc.)
- `GET /actions/possible` — List currently valid actions
- `GET /actions/log` — Full action history for replay/debugging

#### Game State
- `GET /encounter` — Current encounter state
- `GET /encounter/results` — History of encounter outcomes
- `GET /player/tokens` — Current token balances
- `GET /metrics` — Session statistics (win rates, token flows, encounter counts)
- `GET /version` — Game version and configuration fingerprint

#### Library (Card Definitions)
- `GET /library/cards` — All card definitions with effects and costs
- `GET /library/cards/<id>` — Specific card definition
- `GET /library/card_effects/<id>` — Specific card effect
- `GET /library/tokens` — All token type definitions

#### Documentation
- `GET /docs/tutorial` — New-player walkthrough
- `GET /docs/hints` — Per-discipline strategies and tips
- `GET /docs/designer` — Designer reference for encounters, cards, and tokens

### Example: Starting a Game

```bash
# Start a new game with seed 42 (deterministic)
curl -X POST http://localhost:8000/action \
  -H "Content-Type: application/json" \
  -d '{"action_type": "NewGame", "seed": 42}'

# See what actions are available
curl http://localhost:8000/actions/possible

# Check your token balances
curl http://localhost:8000/player/tokens

# Pick an encounter card to start
curl -X POST http://localhost:8000/action \
  -H "Content-Type: application/json" \
  -d '{"action_type": "EncounterPickEncounter", "card_id": 36}'

# View session statistics
curl http://localhost:8000/metrics
```

See `docs/examples/api_examples.sh` for a complete gameplay walkthrough.

## Development

### Seeding and Reproducibility

- Provide a seed when starting a new game: `{"action_type": "NewGame", "seed": 42}`
- The server records every action in the ActionLog (`GET /actions/log`) so runs can be reproduced from seed + action sequence.

### Running Tests

Run the full validation suite (formatting, clippy, build, tests, coverage):
```bash
make check
```

Run only the test suite:
```bash
cargo test
```

Run tests with output:
```bash
cargo test -- --nocapture
```

The `tests/scenario_tests.rs` file contains long-scenario integration tests that exercise full gameplay loops (new game → combat → scout → next encounter). These tests use only production endpoints and serve as a guide for how to play the game via the HTTP API.

### Code Quality

```bash
# All-in-one validation (recommended before every commit)
make check

# Individual checks
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt -- --check
cargo llvm-cov --workspace --fail-under-lines 80
```

### Project Structure

```
configurations/             # JSON game content (embedded at compile time)
├── general/
│   ├── tokens.json         # Initial token balances
│   ├── game_rules.json     # Game-wide mechanics constants
│   └── shared_effects.json # Shared effect templates (damage, shield, etc.)
├── combat/cards.json       # Combat cards, effects, and encounters
├── mining/cards.json
├── herbalism/cards.json
├── woodcutting/cards.json
├── fishing/cards.json
├── rest/cards.json
├── crafting/cards.json
├── research/cards.json
└── milestone/cards.json

src/
├── lib.rs              # Library entry point, route mounting
├── main.rs             # Binary entry point
├── action/             # Player action handling (PlayerActions enum)
├── combat/             # Combat encounter logic and state queries
├── docs/               # Self-documenting API endpoints
│   ├── tutorial.rs     # New-player walkthrough
│   ├── hints.rs        # Per-discipline strategies
│   └── designer.rs     # Designer reference guide
├── library/            # Core domain: types, game state, disciplines
│   ├── types.rs        # All core types (TokenType, CardKind, etc.)
│   ├── config.rs       # JSON-deserializable config types
│   ├── config_loader.rs # Loader that parses JSON and builds Library
│   ├── game_state.rs   # GameState struct, encounter management
│   ├── metrics.rs      # Session metrics computation
│   ├── endpoints.rs    # Library card/token query endpoints
│   └── disciplines/    # Per-discipline encounter logic
├── player_data.rs      # Player state management
├── player_tokens.rs    # Token balance endpoint
├── version.rs          # Version and config hash endpoint
└── status_messages.rs  # API response messages

tests/
├── scenario_tests.rs   # Full gameplay loop integration tests
├── flow_tests.rs       # Combat flow integration tests
└── docs_tests.rs       # Documentation endpoint tests
```

## Game Configuration

All card, effect, and encounter definitions are externalized as JSON files in the `configurations/` directory. Files are embedded at compile time via `include_str!()` — no runtime file I/O is needed.

- **`general/tokens.json`** — Starting token balances (Health, Stamina, Durabilities, max hand sizes)
- **`general/game_rules.json`** — Game-wide mechanics constants (death reset values, combat rewards, crafting costs, scouting parameters, milestone scaling, woodcutting patterns)
- **`general/shared_effects.json`** — 5 shared effect templates reusable across disciplines (deal_damage, grant_shield, grant_stamina, draw_cards, insight)
- **`<discipline>/cards.json`** — Per-discipline cards, effects, and encounters for all 9 disciplines

Each JSON file contains a `"cards"` array with entries tagged by `"type"`: `"effect"` (effect templates), `"player_card"` (cards referencing effects), or `"encounter"` (encounter definitions). Cards reference effects by namespaced name — e.g., `"shared:deal_damage"` or `"combat:enemy_shield"` — where the prefix matches the discipline folder. See `GET /docs/designer` for the full authoring reference.

To modify game content, edit the JSON files and recompile. The config types are defined in `src/library/config.rs` and loaded by `src/library/config_loader.rs`.

## Card States

Cards transition through different states during gameplay:
- **Deck**: Card is in the deck ready to be drawn
- **Hand**: Card has been drawn and is in the player's hand
- **Discarded**: Card has been played and is in the discard pile
- **Deleted**: Card has been removed from the game

## Design Philosophy

- **Encapsulation**: Internal APIs remain private; all interactions go through public HTTP endpoints
- **Type Safety**: Leverages Rust's type system for correctness
- **Self-Documenting**: The API explains itself via `/docs/*` endpoints and rich OpenAPI comments
- **Error Handling**: No panics in production code; all errors return proper HTTP status codes
- **Testing**: Comprehensive integration tests covering all endpoints and edge cases

## Contributing

See `docs/dev/CONTRIBUTING.md` for detailed guidelines. Key principles:
- Zero clippy warnings
- No unwrap() calls in production code
- ≥80% line coverage enforced
- Meaningful commit messages

## Documentation Structure

- `docs/design/` — Vision, roadmap, and current state
  - `vision.md` — High-level design principles and core mechanics
  - `roadmap.md` — Implementation roadmap
  - `current_state.md` — Current implementation status
- `docs/dev/` — Developer guidance
  - `CONTRIBUTING.md` — Code standards and testing expectations
  - `SECURITY.md` — Security audit procedures
- `docs/audits/` — Security and quality audits
- `docs/examples/` — Example API usage scripts

### Developer Setup — Pre-commit Hooks

Install pre-commit (`pip install --user pre-commit`) and enable the hooks:

```bash
make install-hooks
# or
./scripts/install-hooks.sh
```

The hooks run `cargo fmt` (auto-fix) and `cargo clippy` on every commit.

## License

Apache-2.0

## Author

RobbingDaHood
