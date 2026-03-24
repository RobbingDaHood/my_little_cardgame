# Copilot instructions for my_little_cardgame

This file guides Copilot CLI sessions and other assistive agents working on this repository.

Build, test, and lint commands

- **Primary validation command**: `make check` — runs formatting (auto-fix), clippy, build, tests, and coverage (80% threshold) in one pass. Reports all errors at the end.
- Run (development server): `cargo run` (server listens on http://localhost:8000 by default).
- Run a single test by name: `cargo test <test_name>` (substring matching supported).
- Run tests with visible output: `cargo test -- --nocapture`.
- Run coverage only: `cargo llvm-cov --workspace --fail-under-lines 80`.
- Pre-commit hooks auto-run `cargo fmt` (auto-fix) and `cargo clippy` on every commit. Tests are validated via `make check`.
- **All tests and coverage must pass before pushing code.** Never accept or commit known test failures. If a test fails, fix the test or the production code before committing. CI enforces ≥80% line coverage — ensure `make check` passes locally before pushing. If in doubt, ask the repository owner.

Key files and types (quick reference)

- `src/library/types.rs` — all core types: TokenType enum, TokenAmount, CardKind, MiningCardEffect, EncounterState structs, CombatPhase, EncounterOutcome, ActionPayload
- `src/library/game_state.rs` — GameState struct, initialization, token balances, encounter phase management, player death mechanic
- `src/library/disciplines/` — per-discipline modules (combat.rs, mining.rs, herbalism.rs, woodcutting.rs, fishing.rs): encounter logic, card registration, conclude/finish methods
- `src/library/metrics.rs` — session metrics computation and GET /metrics endpoint
- `src/docs/` — self-documenting API endpoints: tutorial.rs, hints.rs, designer.rs
- `src/action/mod.rs` — action handler dispatch (PlayerActions enum match)
- `src/library/endpoints.rs` — HTTP route handlers for library cards, card effects, possible actions
- `src/lib.rs` — library entry point, route mounting
- `src/main.rs` — binary entry, Rocket launch
- `tests/scenario_tests.rs` — integration tests exercising full gameplay loops
- `tests/flow_tests.rs` — combat flow integration tests
- `tests/docs_tests.rs` — integration tests for /docs/* and /metrics endpoints

High-level architecture

- Project is a Rust web API built with Rocket exposing REST endpoints for cards, decks, and combat.
- Core crates and layout:
  - `src/library/` — core domain module: types, game state, combat resolution, encounter loop, token registry, action log, metrics, and HTTP endpoints.
  - `src/docs/` — self-documenting API endpoints: tutorial walkthrough, strategy hints, designer reference.
  - `src/action/` — player action handling and request processing.
  - `src/player_data.rs` — player state and persistence logic.
  - `src/player_tokens.rs` — player token balance endpoint.
  - `src/status_messages.rs` — standardized API response messages.
- All runtime behaviour is exposed via HTTP endpoints; most internal functionality is tested with integration tests that drive the API.

Key conventions and repository-specific notes

- "Everything is a deck" design: core game state is modelled as decks (Attack, Defence, Resource) and cards move between Deck, Hand, Discarded, Deleted states.
- Tests: place tests in separate files under the top-level `tests/` directory (do not put tests inline in `src` files). Prefer integration tests that exercise the public HTTP API (see `tests/` and `src/tests.rs`). Do not make items `pub` solely to enable unit testing — keep as much of the program private as possible and test through integration tests instead. When running a single integration test, use the test name shown in source (substring matching is supported by `cargo test`). Aim for at least 90% test coverage before committing; ensure coverage is measured and enforced in CI.
- Scenario tests: `tests/scenario_tests.rs` contains long-scenario integration tests that exercise full gameplay loops (new game → combat → scout → next encounter). These tests use only production endpoints and serve as living documentation. When adding new encounter types, card mechanics, or gameplay features, update or add scenario tests so they remain an accurate API gameplay guide.
- OpenAPI/Swagger is enabled using `rocket_okapi`; when the server is running, view Swagger UI at `/swagger/`.
- No unwraps and zero Clippy warnings policy: avoid adding unwrap() in production code; prefer Result propagation and explicit error handling.
- Breaking changes are allowed: do not hold back from making breaking changes (API, data format, struct layout, etc.) when they improve the codebase. When a commit includes breaking changes, clearly state "BREAKING:" in the commit summary and list what changed.
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
- Always respect everything written in the files in the docs/ folder; treat those files as authoritative guidance for the repository and follow them without contradiction.

Notes for Copilot sessions

- Prefer reading `README.md` and `src/` modules before making changes; the README contains useful usage and testing commands.
- When adding or changing endpoints, update both `src/lib.rs` and `src/main.rs` and add an integration test under `tests/`.
- Keep changes minimal. 
- Before every commit, run `make check` to validate all checks pass. Pre-commit hooks provide a fast safety net (fmt + clippy) on commit.
- When printing a URL to the console, never wrap it in parentheses or brackets — bare URLs are clickable in the terminal, wrapped ones are not.

Suggest changes to vision.md and roadmap.md

- vision.md and roadmap.md is the authoritative. 
- At the end of any plan suggest improvement to both files and save that in a file. Do not place the file in docs/design.
- The suggestions should be based on new information given or found during planning and execution of the plan. 
- The goal is to keep vision.md and roadmap.md up to date and in high quality. 

Documentation maintenance

All documentation must stay in sync with the code. When making changes, follow these rules:

- **OpenAPI doc comments**: When adding or changing endpoints, update the `///` doc comments on handler functions and action enum variants. Comments should explain *strategic purpose* (why a player or designer would use it), not just restate the function signature.
- **Self-documenting endpoints**: When adding or modifying game mechanics, card effects, encounter types, or disciplines, update the relevant `/docs/*` endpoint content:
  - `src/docs/tutorial.rs` — new-player walkthrough steps
  - `src/docs/hints.rs` — per-discipline strategies, tips, and pitfalls
  - `src/docs/designer.rs` — encounter/card/token/effect authoring reference
- **README.md**: When adding new endpoints, add them to the API endpoint table and describe their purpose. Fix any outdated endpoint references.
- **Examples**: Keep `docs/examples/api_examples.sh` working — update curl commands when endpoints or payloads change. The example should demonstrate a full gameplay loop with current endpoints.
- **CONTRIBUTING.md**: When changing development workflows or conventions, update `docs/dev/CONTRIBUTING.md` accordingly.
- **Metrics**: The `/metrics` endpoint content updates automatically from gameplay data; no manual documentation updates needed for it.
- **Spot-check**: After documentation-related changes, run the server and verify `/swagger/`, `/docs/tutorial`, `/docs/hints`, and `/docs/designer` render correctly.

MCP servers

Would you like to configure any MCP servers (e.g., Playwright for web/API testing) for this repository? If so, specify which servers to configure.

Rate limits 

If you ever get a message about being rate limited then stop the current plan and wait for me to continue the plan later. 

Messages could contain phrases like "rate limit that restricts the number of Copilot model requests" but is not limited to that. 

Do not continue retrying if that message shows up! 

Branches and pull requests

At the start of every plan, ask the user:
1. Should this work be done on a new branch or the current branch?
2. Should a pull request be created at the end?

When creating a new branch, always branch from the latest main branch (fetch and checkout main first).

Always commit small isolated commits, but each commit should pass the tests and other checks.

Always rebase on main before pushing.

When creating a pull request, always write a clear, descriptive PR body that summarizes what changed, why, and any important context for reviewers.

GitHub CLI and git operations

Use `gh` (GitHub CLI) and `git` for **all** repository and GitHub operations:

- **git**: commit, push, pull, rebase, branch, merge, diff, log, status.
- **gh**: create/view PRs (`gh pr create`, `gh pr view`), manage issues (`gh issue`), browse repo (`gh browse`), check CI status (`gh run list`), and any other GitHub interaction.

Authentication:
- `gh` authenticates via the `GH_TOKEN` environment variable (stored in `.env` at the repo root).
- `.env` is in `.gitignore` and must **never** be committed.
- If `GH_TOKEN` is not set in the environment, source it: `export $(cat .env | xargs)` (or instruct the user to set it).

Agents are free to push branches and create pull requests using `gh` and `git`.

Worktree setup for parallel AI work

This repository uses git worktrees to allow multiple AI agents to work in parallel without interfering with each other. Each worktree is an independent working directory with its own branch, sharing the same git history.

Layout:
```
Projects/
  my_little_cardgame/            ← main repo checkout (manual work)
  my_little_cardgames/           ← worktree parent folder
    wt1/                         ← worktree, branch: worktree/wt1
    wt2/                         ← worktree, branch: worktree/wt2
    wt3/                         ← worktree, branch: worktree/wt3
```

How AI agents should use worktrees:
- Each AI session is assigned one worktree directory (e.g., `my_little_cardgames/wt1`).
- The new folder should be named something similar to the branch. 
- Detect which worktree you are in by checking the current working directory.
- Create feature branches from the worktree branch as usual (branch from latest `origin/main`).
- Each worktree has its own `target/` build directory — builds are fully independent.
- Use `git push` and `gh pr create` from worktrees just like from the main checkout.

Managing worktrees with `scripts/worktree-manage.sh`:
- `scripts/worktree-manage.sh list` — list all worktrees.
- `scripts/worktree-manage.sh add <name>` — create a new worktree from latest `origin/main`.
- `scripts/worktree-manage.sh remove <name>` — remove a worktree and its branch.
- `scripts/worktree-manage.sh reset <name>` — hard-reset a worktree to latest `origin/main` (clean slate).

Run the script from the main repo or any worktree — it resolves paths automatically.
