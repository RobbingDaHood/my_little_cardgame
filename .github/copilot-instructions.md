# Copilot instructions for my_little_cardgame

This file guides Copilot CLI sessions and other assistive agents working on this repository.

Build, test, and lint commands

- Build: `cargo build --release` (or `cargo build` for dev).
- Run (development server): `cargo run` (server listens on http://localhost:8000 by default).
- Run full test suite: `cargo test`.
- Run a single test by name: `cargo test <test_name>` (use a substring of the test function name).
- Run tests with visible output: `cargo test -- --nocapture`.
- Lint with Clippy: `cargo clippy --all-targets --all-features -- -D warnings`. Run this command before each commit.
- Format: `cargo fmt`.
- Ensure formatting checks pass before committing: `cargo fmt -- --check`.

High-level architecture

- Project is a Rust web API built with Rocket exposing REST endpoints for cards, decks, and combat.
- Core crates and layout:
  - `src/lib.rs` — library entry point exposing the public API used by the binary.
  - `src/main.rs` — binary entry that mounts Rocket routes and serves the OpenAPI/Swagger UI.
  - `src/deck/` — deck and card management (card definitions, deck operations, token handling).
  - `src/combat/` — combat resolution and play logic.
  - `src/action/` — player action handling and request processing.
  - `src/player_data.rs` — player state and persistence logic.
  - `src/status_messages.rs` — standardized API response messages.
- All runtime behaviour is exposed via HTTP endpoints; most internal functionality is tested with integration tests that drive the API.

Key conventions and repository-specific notes

- "Everything is a deck" design: core game state is modelled as decks (Attack, Defence, Resource) and cards move between Deck, Hand, Discarded, Deleted states.
- Tests: place tests in separate files under the top-level `tests/` directory (do not put tests inline in `src` files). Prefer integration tests that exercise the public HTTP API (see `tests/` and `src/tests.rs`). Do not make items `pub` solely to enable unit testing — keep as much of the program private as possible and test through integration tests instead. When running a single integration test, use the test name shown in source (substring matching is supported by `cargo test`). Aim for at least 90% test coverage before committing; ensure coverage is measured and enforced in CI.
- OpenAPI/Swagger is enabled using `rocket_okapi`; when the server is running, view Swagger UI at `/swagger/`.
- No unwraps and zero Clippy warnings policy: avoid adding unwrap() in production code; prefer Result propagation and explicit error handling.
- Features and dependencies: Rocket is built with `json` feature disabled by default — follow existing Cargo.toml features when adding dependencies.
- Prefer simpler code wrapped in well-named wrapper methods instead of relying on long explanatory comments; remove obvious comments that merely restate what clear function/variable names communicate. Favor expressive names and small helper functions over comment-heavy implementations.
 - Consider using Rust enums for discrete states or variant data (e.g., deck or card states); prefer enums over ad-hoc strings or booleans when it improves clarity, type-safety, and enables exhaustive matching.

  - When to use enums vs newtypes vs strings:
    - Use enums for closed sets of variants (CardType, CardState, TokenLifecycle).
    - Use newtype wrappers (e.g., struct TokenId(String)) when the value is opaque but needs stronger typing.
    - Use plain strings only for truly dynamic, designer-driven values.

  - Examples:
    - CardType: derive Serialize/Deserialize/JsonSchema and use in API structs:
      ```rust
      #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
      #[serde(crate = "rocket::serde")]
      pub enum CardType { Attack, Defence, Resource }
      ```
    - TokenId/newtype:
      ```rust
      #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
      #[serde(transparent, crate = "rocket::serde")]
      pub struct TokenId(pub String);
      ```

  - Implementation notes for agents:
    - Prefer returning typed `Json<T>` from handlers and deriving JsonSchema so OpenAPI is accurate.
    - Avoid building JSON strings by hand (RawJson); map domain types to serde-serializable structs instead.
    - For action payloads, prefer structured payloads (typed serde enums) instead of pipe-separated strings; prefer a strongly-typed serde enum (derive Serialize/Deserialize/JsonSchema) and use `serde_json::Value` only as a short-term fallback when necessary.

Files to check for agent config

- Existing repo files inspected: README.md, Cargo.toml, src/.
- If present, include and merge guidance from: CLAUDE.md, AGENTS.md, CONVENTIONS.md, AIDER_CONVENTIONS.md, .cursorrules, .cursor/, .windsurfrules, .clinerules, .cline_rules. (None were found at time of creation.)
- Always respect everything written in the files in the docks/ folder; treat those files as authoritative guidance for the repository and follow them without contradiction.

Notes for Copilot sessions

- Prefer reading `README.md` and `src/` modules before making changes; the README contains useful usage and testing commands.
- When adding or changing endpoints, update both `src/lib.rs` and `src/main.rs` and add an integration test under `tests/`.
- Keep changes minimal and run `cargo test` and `cargo clippy --all-targets --all-features -- -D warnings` before opening PRs.

MCP servers

Would you like to configure any MCP servers (e.g., Playwright for web/API testing) for this repository? If so, specify which servers to configure.
